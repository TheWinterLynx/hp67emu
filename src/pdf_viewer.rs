//! Embedded desktop PDF viewer. Rendering stays off the live-machine/UI thread.
use eframe::egui;
use hayro::{hayro_interpret::InterpreterSettings, hayro_syntax::Pdf, RenderCache, RenderSettings};
use std::sync::{mpsc, Arc};

include!("../documentation/embedded.rs");

#[derive(Clone)]
pub struct ViewRequest {
    pub file: &'static str,
    pub title: String,
    pub page: usize,
}

pub fn request(ctx: &egui::Context, file: &'static str, title: String, page: usize) {
    ctx.request_repaint_of(egui::ViewportId::ROOT);
    ctx.data_mut(|data| {
        data.insert_temp(
            egui::Id::new("hp67-pdf-request"),
            ViewRequest { file, title, page },
        )
    });
}

pub fn has_pdf(file: &str) -> bool {
    embedded(file).is_some()
}

fn embedded(file: &str) -> Option<&'static [u8]> {
    EMBEDDED_PDFS
        .iter()
        .find(|(name, _)| *name == file)
        .map(|(_, bytes)| *bytes)
}

#[derive(Clone, Copy)]
struct RenderRequest {
    file: &'static str,
    page: usize,
    width: u16,
    generation: u64,
}
struct Rendered {
    generation: u64,
    page: usize,
    pages: usize,
    image: Result<egui::ColorImage, String>,
}
struct Worker {
    send: mpsc::SyncSender<RenderRequest>,
    receive: mpsc::Receiver<Rendered>,
}

impl Worker {
    fn new(ctx: egui::Context) -> Self {
        let (send, receive) = mpsc::sync_channel::<RenderRequest>(1);
        let (result_send, result_receive) = mpsc::sync_channel::<Rendered>(1);
        std::thread::spawn(move || {
            let Ok(mut next) = receive.recv() else {
                return;
            };
            loop {
                let file = next.file;
                // Arc wraps a static slice: opening a manual does not clone its PDF bytes.
                let parsed = embedded(file)
                    .ok_or_else(|| "PDF no incluido".to_owned())
                    .and_then(|bytes| {
                        Pdf::new(Arc::new(bytes)).map_err(|e| format!("PDF inválido: {e:?}"))
                    });
                let mut cache = RenderCache::new();
                let mut cached_page = next.page;
                loop {
                    while let Ok(newer) = receive.try_recv() {
                        next = newer;
                    }
                    if next.file != file {
                        break;
                    }
                    if next.page != cached_page {
                        cache = RenderCache::new();
                        cached_page = next.page;
                    }
                    let pages = parsed.as_ref().map(|pdf| pdf.pages().len()).unwrap_or(0);
                    let image = match &parsed {
                        Ok(pdf) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            render_page(pdf, &cache, next.page, next.width)
                        }))
                        .unwrap_or_else(|_| {
                            Err("El renderizador no pudo mostrar esta página".to_owned())
                        }),
                        Err(error) => Err(error.clone()),
                    };
                    if result_send
                        .send(Rendered {
                            generation: next.generation,
                            page: next.page,
                            pages,
                            image,
                        })
                        .is_err()
                    {
                        return;
                    }
                    ctx.request_repaint();
                    let Ok(request) = receive.recv() else {
                        return;
                    };
                    next = request;
                    if next.file != file {
                        break;
                    }
                }
            }
        });
        Self {
            send,
            receive: result_receive,
        }
    }
}

fn render_page<'a>(
    pdf: &'a Pdf,
    cache: &RenderCache<'a>,
    page: usize,
    width: u16,
) -> Result<egui::ColorImage, String> {
    let page = pdf
        .pages()
        .get(page)
        .ok_or_else(|| "La sección apunta fuera del manual".to_owned())?;
    let (w, h) = page.render_dimensions();
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return Err("Tamaño de página inválido".to_owned());
    }
    let scale = (f32::from(width.clamp(320, 2200)) / w).min(3000.0 / h);
    let settings = RenderSettings {
        x_scale: scale,
        y_scale: scale,
        bg_color: hayro::vello_cpu::color::palette::css::WHITE,
        ..Default::default()
    };
    let pixmap = hayro::render(page, cache, &InterpreterSettings::default(), &settings);
    Ok(egui::ColorImage::from_rgba_premultiplied(
        [usize::from(pixmap.width()), usize::from(pixmap.height())],
        pixmap.data_as_u8_slice(),
    ))
}

#[derive(Default)]
pub struct PdfViewer {
    worker: Option<Worker>,
    pending: Option<RenderRequest>,
    open: bool,
    file: Option<&'static str>,
    title: String,
    page: usize,
    anchor: usize,
    pages: usize,
    zoom: f32,
    generation: u64,
    busy: bool,
    texture: Option<egui::TextureHandle>,
    error: Option<String>,
}

impl PdfViewer {
    fn queue(&mut self, ctx: &egui::Context) {
        let Some(file) = self.file else {
            return;
        };
        self.generation = self.generation.wrapping_add(1);
        self.busy = true;
        self.error = None;
        self.texture = None;
        let width = (900.0 * self.zoom * ctx.pixels_per_point()).clamp(320.0, 2200.0) as u16;
        self.pending = Some(RenderRequest {
            file,
            page: self.page,
            width,
            generation: self.generation,
        });
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let request = ctx.data_mut(|data| {
            let id = egui::Id::new("hp67-pdf-request");
            let request = data.get_temp::<ViewRequest>(id);
            data.remove::<ViewRequest>(id);
            request
        });
        if let Some(request) = request {
            self.file = Some(request.file);
            self.title = request.title;
            self.page = request.page;
            self.anchor = request.page;
            self.pages = 0;
            self.zoom = 1.0;
            self.open = true;
            if self.worker.is_none() {
                self.worker = Some(Worker::new(ctx.clone()));
            }
            self.queue(ctx);
        }
        if let Some(worker) = &self.worker {
            while let Ok(result) = worker.receive.try_recv() {
                if result.generation != self.generation {
                    continue;
                }
                self.pages = result.pages;
                self.page = result.page;
                self.busy = false;
                match result.image {
                    Ok(image) if self.open => {
                        self.texture = Some(ctx.load_texture(
                            "hp67-pdf-page",
                            image,
                            egui::TextureOptions::LINEAR,
                        ))
                    }
                    Ok(_) => {}
                    Err(error) => self.error = Some(error),
                }
            }
            if let Some(request) = self.pending.take() {
                match worker.send.try_send(request) {
                    Ok(()) => {}
                    Err(mpsc::TrySendError::Full(request)) => self.pending = Some(request),
                    Err(mpsc::TrySendError::Disconnected(_)) => {
                        self.error = Some("El visor PDF no está disponible".to_owned());
                        self.busy = false;
                    }
                }
            }
        }
        if !self.open {
            return;
        }
        let mut open = self.open;
        let mut changed = false;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("hp67-pdf-viewer"),
            egui::ViewportBuilder::default()
                .with_title("HP-67 · Instrucciones")
                .with_inner_size([900.0, 700.0])
                .with_min_inner_size([420.0, 300.0]),
            |ctx, class| {
                if ctx.input(|input| input.viewport().close_requested()) {
                    open = false;
                }
                let mut contents = |ui: &mut egui::Ui| {
                    ui.heading(&self.title);
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .add_enabled(self.page > 0, egui::Button::new("◀ Anterior"))
                            .clicked()
                        {
                            self.page -= 1;
                            changed = true;
                        }
                        ui.label(format!("Página PDF {} / {}", self.page + 1, self.pages));
                        if ui
                            .add_enabled(
                                self.pages > 0 && self.page + 1 < self.pages,
                                egui::Button::new("Siguiente ▶"),
                            )
                            .clicked()
                        {
                            self.page += 1;
                            changed = true;
                        }
                        if ui.button("Inicio del programa").clicked() {
                            self.page = self.anchor;
                            changed = true;
                        }
                        if ui.button("−").clicked() {
                            self.zoom = (self.zoom / 1.25).max(0.5);
                            changed = true;
                        }
                        ui.label(format!("{:.0}%", self.zoom * 100.0));
                        if ui.button("+").clicked() {
                            self.zoom = (self.zoom * 1.25).min(3.0);
                            changed = true;
                        }
                    });
                    ui.separator();
                    if self.busy {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Preparando página…");
                        });
                    }
                    if let Some(error) = &self.error {
                        ui.colored_label(egui::Color32::LIGHT_RED, error);
                    }
                    if let Some(texture) = &self.texture {
                        egui::ScrollArea::both()
                            .id_source(("hp67-pdf-scroll", self.file, self.page))
                            .show(ui, |ui| {
                                ui.image((
                                    texture.id(),
                                    texture.size_vec2() / ctx.pixels_per_point(),
                                ));
                            });
                    }
                };
                if class == egui::ViewportClass::Embedded {
                    egui::Window::new("HP-67 · Instrucciones")
                        .open(&mut open)
                        .default_size([900.0, 700.0])
                        .show(ctx, &mut contents);
                } else {
                    egui::CentralPanel::default().show(ctx, &mut contents);
                }
            },
        );
        self.open = open;
        if !open {
            self.texture = None;
            self.pending = None;
            self.worker = None;
            self.busy = false;
        }
        if changed && open {
            self.queue(ctx);
        }
        if self.busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}

/// Portable smoke check: operates only on bytes linked into this executable.
pub fn verify_embedded() -> Result<usize, String> {
    let mut counts = std::collections::BTreeMap::new();
    for (file, bytes) in EMBEDDED_PDFS {
        let pdf = Pdf::new(Arc::new(*bytes)).map_err(|error| format!("{file}: {error:?}"))?;
        counts.insert(*file, pdf.pages().len());
        let cache = RenderCache::new();
        let page = crate::documentation::program_sections()
            .iter()
            .filter(|(_, name, _)| name == file)
            .map(|(_, _, page)| page - 1)
            .min()
            .unwrap_or(0);
        let rendered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_page(&pdf, &cache, page, 320)
        }))
        .map_err(|_| format!("{file}: renderizador interrumpido"))??;
        if rendered.pixels.is_empty() {
            return Err(format!("{file}: página vacía"));
        }
    }
    for (reference, file, page) in crate::documentation::program_sections() {
        if *page == 0 || *page > counts.get(file).copied().unwrap_or(0) {
            return Err(format!("{reference}: sección fuera del PDF {file}"));
        }
    }
    Ok(EMBEDDED_PDFS.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_manual_is_embedded_as_a_real_pdf() {
        assert_eq!(EMBEDDED_PDFS.len(), 12);
        for (name, bytes) in EMBEDDED_PDFS {
            assert!(bytes.starts_with(b"%PDF-"), "{name}");
        }
    }
    #[test]
    #[ignore = "PDF raster smoke test; run in the owner local gate"]
    fn embedded_standard_instructions_render_without_external_files() {
        let bytes = embedded("hp67-pac-standard-en.pdf").unwrap();
        let pdf = Pdf::new(Arc::new(bytes)).unwrap();
        let cache = RenderCache::new();
        let image = render_page(&pdf, &cache, 9, 480).unwrap();
        assert!(image.size[0] > 100 && image.size[1] > 100);
        assert!(image.pixels.windows(2).any(|pair| pair[0] != pair[1]));
        assert!(render_page(&pdf, &cache, pdf.pages().len(), 480).is_err());
    }
}
