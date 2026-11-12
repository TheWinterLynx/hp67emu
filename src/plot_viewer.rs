//! Desktop chart presentation for points sampled by the real HP-67 firmware.

use std::{fmt::Write, sync::mpsc, thread};

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};

use crate::hp67::plot_sampler::PolynomialSampler;

const CYAN: Color32 = Color32::from_rgb(79, 225, 214);
const MUTED: Color32 = Color32::from_rgb(146, 162, 180);
const BG: Color32 = Color32::from_rgb(13, 19, 29);
const GRID: Color32 = Color32::from_rgb(37, 50, 67);

enum Update {
    Point([f64; 2]),
    Done(Result<(), String>),
}

pub(crate) struct PlotViewer {
    pub(crate) open: bool,
    coefficients: [f64; 4],
    limits: [f64; 2],
    samples: usize,
    receiver: Option<mpsc::Receiver<Update>>,
    points: Vec<[f64; 2]>,
    sampled_limits: [f64; 2],
    status: String,
    complete: bool,
}

impl Default for PlotViewer {
    fn default() -> Self {
        Self {
            open: false,
            coefficients: [-9140.0, -7.596, 4.243e-3, -0.742e-6],
            limits: [0.0, 1_000.0],
            samples: 101,
            receiver: None,
            points: Vec::new(),
            sampled_limits: [0.0, 1_000.0],
            status: "Pulsa Dibujar para calcular la curva de ejemplo.".into(),
            complete: false,
        }
    }
}

impl PlotViewer {
    fn start(&mut self, ctx: &egui::Context) {
        let [min, max] = self.limits;
        let coefficients = self.coefficients;
        if !min.is_finite()
            || !max.is_finite()
            || min >= max
            || min.abs().max(max.abs()) > 1e6
            || coefficients
                .iter()
                .any(|&v| !v.is_finite() || v.abs() > 1e20 || (v != 0.0 && v.abs() < 1e-20))
        {
            self.status = "Comprueba el intervalo (±1 000 000) y los coeficientes.".into();
            return;
        }
        let count = self.samples;
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        let work = thread::Builder::new()
            .name("hp67-plot".into())
            .spawn(move || {
                let result = (|| {
                    let mut machine = PolynomialSampler::new(coefficients)?;
                    for i in 0..count {
                        let x = if i == count - 1 {
                            max
                        } else {
                            min + (max - min) * i as f64 / (count - 1) as f64
                        };
                        let point = machine
                            .sample(x)
                            .map_err(|error| format!("x = {x}: {error}"))?;
                        if sender.send(Update::Point(point)).is_err() {
                            return Ok(());
                        }
                        ctx.request_repaint();
                    }
                    Ok(())
                })();
                let _ = sender.send(Update::Done(result));
                ctx.request_repaint();
            });
        match work {
            Ok(_) => {
                self.receiver = Some(receiver);
                self.points.clear();
                self.sampled_limits = [min, max];
                self.complete = false;
                self.status = "Arrancando firmware y leyendo SD1-09A…".into();
            }
            Err(error) => self.status = format!("No se pudo iniciar el cálculo: {error}"),
        }
    }

    fn poll(&mut self) {
        let Some(receiver) = &self.receiver else {
            return;
        };
        loop {
            match receiver.try_recv() {
                Ok(Update::Point(point)) => {
                    self.points.push(point);
                    self.status = format!("Calculando: {} / {}", self.points.len(), self.samples);
                }
                Ok(Update::Done(result)) => {
                    self.complete = result.is_ok();
                    self.status = match result {
                        Ok(()) => format!("{} muestras · pantalla HP-67", self.points.len()),
                        Err(error) => format!("Cálculo detenido: {error}"),
                    };
                    self.receiver = None;
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = "La tarea de cálculo terminó sin devolver un resultado.".into();
                    self.receiver = None;
                    break;
                }
            }
        }
    }

    pub(crate) fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        self.poll();
        let mut open = self.open;
        egui::Window::new("HP-67 · Visor gráfico")
            .open(&mut open)
            .default_size([720.0, 500.0])
            .min_width(330.0)
            .vscroll(true)
            .frame(egui::Frame::window(&ctx.style()).fill(BG))
            .show(ctx, |ui| {
                use egui::TextStyle::{Body, Button, Heading, Monospace, Small};

                let style = ui.style_mut();
                for (role, size) in [
                    (Body, 16.0),
                    (Button, 16.0),
                    (Heading, 23.0),
                    (Monospace, 15.0),
                    (Small, 14.0),
                ] {
                    if let Some(font) = style.text_styles.get_mut(&role) {
                        font.size = size;
                    }
                }
                ui.spacing_mut().interact_size.y = 24.0;
                ui.label(
                    egui::RichText::new("STANDARD PAC  /  SD1-09A")
                        .color(CYAN)
                        .monospace(),
                );
                ui.heading("Evaluación de polinomios");
                ui.label(egui::RichText::new("y = a₀ + a₁x + a₂x² + a₃x³").color(MUTED));
                let running = self.receiver.is_some();
                ui.add_enabled_ui(!running, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (name, value) in ["a₀", "a₁", "a₂", "a₃"]
                            .into_iter()
                            .zip(&mut self.coefficients)
                        {
                            ui.label(name);
                            ui.add(
                                egui::DragValue::new(value)
                                    .custom_formatter(|value, _| coefficient_label(value))
                                    .speed(0.01),
                            );
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Desde");
                        ui.add(egui::DragValue::new(&mut self.limits[0]).max_decimals(8));
                        ui.label("Hasta");
                        ui.add(egui::DragValue::new(&mut self.limits[1]).max_decimals(8));
                        ui.label("Muestras");
                        ui.add(egui::DragValue::new(&mut self.samples).clamp_range(2..=201));
                        if ui
                            .button(egui::RichText::new("Dibujar").color(CYAN))
                            .clicked()
                        {
                            self.start(ctx);
                        }
                    });
                });

                let cursor = chart(ui, &self.points, self.sampled_limits);
                ui.horizontal_wrapped(|ui| {
                    ui.label(&self.status);
                    if running && ui.button("Cancelar").clicked() {
                        self.receiver = None;
                        self.status = "Cálculo cancelado.".into();
                    }
                    if ui
                        .add_enabled(self.complete, egui::Button::new("Copiar CSV"))
                        .clicked()
                    {
                        let mut csv = String::from("x,y\n");
                        for [x, y] in &self.points {
                            let _ = writeln!(csv, "{x},{y}");
                        }
                        ctx.output_mut(|out| out.copied_text = csv);
                    }
                });
                if let Some([x, y]) = cursor {
                    ui.monospace(format!("x = {x:.8}   y = {y:.8}"));
                } else {
                    ui.label(
                        egui::RichText::new(
                            "Pasa el cursor sobre la gráfica para leer una muestra.",
                        )
                        .color(MUTED),
                    );
                }
            });
        self.open = open;
        if !open {
            if self.receiver.take().is_some() {
                self.status = "Cálculo cancelado.".into();
            }
        }
    }
}

fn coefficient_label(value: f64) -> String {
    let magnitude = value.abs();
    if magnitude != 0.0 && !(0.001..1e6).contains(&magnitude) {
        format!("{value:e}")
    } else {
        value.to_string()
    }
}

fn axis_label(value: f64) -> String {
    let magnitude = value.abs();
    if magnitude != 0.0 && !(0.001..1e5).contains(&magnitude) {
        format!("{value:.2e}")
    } else {
        format!("{value:.3}")
    }
}

fn chart(ui: &mut egui::Ui, points: &[[f64; 2]], [x0, x1]: [f64; 2]) -> Option<[f64; 2]> {
    let height = (ui.ctx().screen_rect().height() * 0.36).clamp(235.0, 410.0);
    let (response, painter) = ui.allocate_painter(
        Vec2::new(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    painter.rect_filled(rect, 7.0, Color32::from_rgb(8, 13, 22));
    let area = Rect::from_min_max(
        rect.min + Vec2::new(110.0, 18.0),
        rect.max - Vec2::new(42.0, 37.0),
    );
    let (mut y0, mut y1) = if points.is_empty() {
        (-1.0, 1.0)
    } else {
        points
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), point| {
                (lo.min(point[1]), hi.max(point[1]))
            })
    };
    let pad = if y0 == y1 {
        (y0.abs() * 0.05).max(0.01)
    } else {
        (y1 - y0) * 0.08
    };
    y0 -= pad;
    y1 += pad;
    let map = |[x, y]: [f64; 2]| {
        Pos2::new(
            area.left() + ((x - x0) / (x1 - x0)) as f32 * area.width(),
            area.bottom() - ((y - y0) / (y1 - y0)) as f32 * area.height(),
        )
    };
    let font = FontId::monospace(14.0);
    let divisions = if area.width() >= 420.0 { 4 } else { 2 };
    for index in 0..=divisions {
        let t = f64::from(index) / f64::from(divisions);
        let x = x0 + (x1 - x0) * t;
        let y = y0 + (y1 - y0) * t;
        let p = map([x, y]);
        painter.line_segment(
            [Pos2::new(p.x, area.top()), Pos2::new(p.x, area.bottom())],
            Stroke::new(1.0_f32, GRID),
        );
        painter.line_segment(
            [Pos2::new(area.left(), p.y), Pos2::new(area.right(), p.y)],
            Stroke::new(1.0_f32, GRID),
        );
        painter.text(
            Pos2::new(p.x, area.bottom() + 8.0),
            Align2::CENTER_TOP,
            axis_label(x),
            font.clone(),
            MUTED,
        );
        painter.text(
            Pos2::new(area.left() - 8.0, p.y),
            Align2::RIGHT_CENTER,
            axis_label(y),
            font.clone(),
            MUTED,
        );
    }
    let plot = painter.with_clip_rect(area);
    if points.len() >= 2 {
        let line: Vec<_> = points.iter().copied().map(map).collect();
        plot.add(egui::Shape::line(
            line.clone(),
            Stroke::new(5.0_f32, CYAN.gamma_multiply(0.16)),
        ));
        plot.add(egui::Shape::line(line, Stroke::new(2.0_f32, CYAN)));
    } else if let Some(&point) = points.first() {
        plot.circle_filled(map(point), 3.0, CYAN);
    }
    let pointer = response.hover_pos().filter(|pos| area.contains(*pos))?;
    let nearest = points.iter().copied().min_by(|&a, &b| {
        (map(a).x - pointer.x)
            .abs()
            .total_cmp(&(map(b).x - pointer.x).abs())
    })?;
    let p = map(nearest);
    plot.line_segment(
        [Pos2::new(p.x, area.top()), Pos2::new(p.x, area.bottom())],
        Stroke::new(1.0_f32, MUTED),
    );
    plot.circle_filled(p, 4.0, CYAN);
    Some(nearest)
}
