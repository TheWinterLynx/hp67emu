//! Plot values sent from the active HP-67 display.
use std::fmt::Write;

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};

use crate::hp67::{plot_sampler::display_number, HardwareDisplayFrame};

const CYAN: Color32 = Color32::from_rgb(79, 225, 214);
const MUTED: Color32 = Color32::from_rgb(146, 162, 180);
const BG: Color32 = Color32::from_rgb(13, 19, 29);
const GRID: Color32 = Color32::from_rgb(37, 50, 67);

const MAX_POINTS: usize = 16_384;

#[derive(Default)]
pub(crate) struct PlotViewer {
    pub(crate) open: bool,
    points: Vec<[f64; 2]>,
    error: Option<&'static str>,
}

impl PlotViewer {
    pub(crate) fn plot(&mut self, frame: Option<HardwareDisplayFrame>) {
        self.open = true;
        if self.points.len() == MAX_POINTS {
            self.error = Some("Serie completa. Copia el CSV o limpia la gráfica.");
            return;
        }
        match frame.map(display_number) {
            Some(Ok(value)) if value.is_finite() => {
                self.points.push([(self.points.len() + 1) as f64, value]);
                self.error = None;
            }
            Some(_) => self.error = Some("No hay un número visible para dibujar."),
            None => self.error = Some("PLOT necesita la calculadora encendida y en RUN."),
        }
    }

    pub(crate) fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("hp67-plot-viewport"),
            egui::ViewportBuilder::default()
                .with_title("HP-67 · Gráficos")
                .with_inner_size([800.0, 540.0])
                .with_min_inner_size([360.0, 320.0]),
            |ctx, _| {
                if ctx.input(|input| input.viewport().close_requested()) {
                    self.open = false;
                    return;
                }
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).fill(BG))
                    .show(ctx, |ui| self.contents(ui));
            },
        );
    }

    fn contents(&mut self, ui: &mut egui::Ui) {
        use egui::TextStyle::{Body, Button, Heading, Monospace};

        let style = ui.style_mut();
        for (role, size) in [
            (Body, 16.0),
            (Button, 16.0),
            (Heading, 23.0),
            (Monospace, 15.0),
        ] {
            if let Some(font) = style.text_styles.get_mut(&role) {
                font.size = size;
            }
        }
        ui.spacing_mut().interact_size.y = 24.0;
        ui.horizontal_wrapped(|ui| {
            ui.heading("Gráfica");
            ui.label(
                egui::RichText::new("PANTALLA HP-67")
                    .monospace()
                    .color(CYAN),
            );
            let has_points = !self.points.is_empty();
            if ui
                .add_enabled(has_points, egui::Button::new("Copiar CSV"))
                .clicked()
            {
                let mut csv = String::from("muestra,valor\n");
                for [x, y] in &self.points {
                    let _ = writeln!(csv, "{x},{y}");
                }
                ui.ctx().output_mut(|out| out.copied_text = csv);
            }
            if ui
                .add_enabled(has_points, egui::Button::new("Limpiar"))
                .clicked()
            {
                self.points.clear();
                self.error = None;
            }
        });
        let limits = if self.points.len() == 1 {
            [0.0, 2.0]
        } else {
            [1.0, self.points.len().max(2) as f64]
        };
        let cursor = chart(ui, &self.points, limits);
        if let Some(error) = self.error {
            ui.colored_label(Color32::LIGHT_RED, error);
        } else if let Some([x, y]) = cursor {
            ui.monospace(format!("Muestra {x:.0}   ·   {y}"));
        } else if self.points.is_empty() {
            ui.label("Pulsa PLOT en la calculadora para añadir el valor de pantalla.");
        } else {
            ui.label(format!(
                "{} muestras · X: orden de captura",
                self.points.len()
            ));
        }
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
    let height = (ui.available_height() - 48.0).max(180.0);
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
