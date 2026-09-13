//! Offline capture of the production egui meshes; no window or GPU required.
use eframe::egui::{self, epaint::Primitive, Color32, Pos2, Rect, Vec2};
use std::{fs, io::Write};

#[test]
#[ignore = "writes canonical visual comparison artifacts"]
fn capture_panel() {
    let directory =
        std::env::var("HP67_CAPTURE_DIR").unwrap_or_else(|_| "target/visual/final".into());
    fs::create_dir_all(&directory).unwrap();
    for scale in [1, 2] {
        let (w, h) = (330 * scale, 620 * scale);
        let ctx = egui::Context::default();
        let mut atlas = Vec::new();
        let mut aw = 0;
        let mut ah = 0;
        // Warm up fonts and layout before capturing a deterministic second frame.
        for frame in 0..4 {
            let mut events = Vec::new();
            if let Ok(id) = std::env::var("HP67_CAPTURE_KEY") {
                let key = crate::ui::geometry::KEYS
                    .iter()
                    .find(|k| k.id == id)
                    .expect("unknown capture key");
                let pos = Pos2::new(key.cx * scale as f32, (key.y + 5.0) * scale as f32);
                events.push(egui::Event::PointerMoved(pos));
                if frame == 1 {
                    events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(w as f32, h as f32),
                    )),
                    time: Some(frame as f64 / 10.0),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none())
                        .show(ctx, |ui| {
                            crate::panel::Hp67Panel::show(ui, &crate::hp67::Hp67State::default());
                        });
                },
            );
            for (_, delta) in &output.textures_delta.set {
                let pixels: Vec<Color32> = match &delta.image {
                    egui::ImageData::Font(f) => f.srgba_pixels(None).collect(),
                    egui::ImageData::Color(c) => c.pixels.clone(),
                };
                let size = delta.image.size();
                if let Some([x, y]) = delta.pos {
                    for row in 0..size[1] {
                        atlas[(y + row) * aw + x..(y + row) * aw + x + size[0]]
                            .copy_from_slice(&pixels[row * size[0]..(row + 1) * size[0]]);
                    }
                } else {
                    aw = size[0];
                    ah = size[1];
                    atlas = pixels;
                }
            }
            if frame < 3 {
                continue;
            }
            let mut rgb = vec![[17.0f32, 18.0, 16.0]; w * h];
            for primitive in ctx.tessellate(output.shapes, 1.0) {
                let Primitive::Mesh(mesh) = primitive.primitive else {
                    panic!("unsupported callback");
                };
                for tri in mesh.indices.chunks_exact(3) {
                    let v = [
                        mesh.vertices[tri[0] as usize],
                        mesh.vertices[tri[1] as usize],
                        mesh.vertices[tri[2] as usize],
                    ];
                    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
                    let area = cross(v[1].pos - v[0].pos, v[2].pos - v[0].pos);
                    if area.abs() < 0.00001 {
                        continue;
                    }
                    let bounds = Rect::from_points(&[v[0].pos, v[1].pos, v[2].pos])
                        .intersect(primitive.clip_rect);
                    for y in (bounds.min.y.floor().max(0.0) as usize)
                        ..(bounds.max.y.ceil().min(h as f32) as usize)
                    {
                        for x in (bounds.min.x.floor().max(0.0) as usize)
                            ..(bounds.max.x.ceil().min(w as f32) as usize)
                        {
                            let p = Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
                            let a = cross(v[1].pos - p, v[2].pos - p) / area;
                            let b = cross(v[2].pos - p, v[0].pos - p) / area;
                            let c = 1.0 - a - b;
                            if a < 0.0 || b < 0.0 || c < 0.0 {
                                continue;
                            }
                            let weights = [a, b, c];
                            let uv = v[0].uv.to_vec2() * a
                                + v[1].uv.to_vec2() * b
                                + v[2].uv.to_vec2() * c;
                            let tx = (uv.x * aw as f32 - 0.5).clamp(0.0, (aw - 1) as f32);
                            let ty = (uv.y * ah as f32 - 0.5).clamp(0.0, (ah - 1) as f32);
                            let (ix, iy) = (tx as usize, ty as usize);
                            let mut tex = [0.0; 4];
                            for (xx, wx) in
                                [(ix, 1.0 - tx.fract()), ((ix + 1).min(aw - 1), tx.fract())]
                            {
                                for (yy, wy) in
                                    [(iy, 1.0 - ty.fract()), ((iy + 1).min(ah - 1), ty.fract())]
                                {
                                    for (k, t) in tex.iter_mut().enumerate() {
                                        *t += atlas[yy * aw + xx].to_array()[k] as f32 / 255.0
                                            * wx
                                            * wy;
                                    }
                                }
                            }
                            let mut color = [0.0; 4];
                            for k in 0..4 {
                                for i in 0..3 {
                                    color[k] +=
                                        v[i].color.to_array()[k] as f32 * weights[i] * tex[k];
                                }
                            }
                            for (k, dst) in rgb[y * w + x].iter_mut().enumerate() {
                                *dst = color[k] + *dst * (1.0 - color[3] / 255.0);
                            }
                        }
                    }
                }
            }
            let mut file = fs::File::create(format!("{directory}/{w}x{h}.ppm")).unwrap();
            write!(file, "P6\n{w} {h}\n255\n").unwrap();
            file.write_all(
                &rgb.iter()
                    .flat_map(|p| p.iter().map(|v| v.round().clamp(0.0, 255.0) as u8))
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        }
    }
}
