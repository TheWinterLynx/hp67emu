//! Analytic surface lighting and deterministic vector microtexture, never a bitmap.
use super::geometry::Transform;
use eframe::egui::{
    epaint::{Mesh, Vertex, WHITE_UV},
    Color32, Painter, Shape, Stroke,
};

pub fn shade(base: Color32, amount: f32) -> Color32 {
    let channel = |v: u8| (v as f32 + amount).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(channel(base.r()), channel(base.g()), channel(base.b()))
}

fn noise(x: u32, y: u32, seed: u32) -> f32 {
    let mut n = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        ^ seed;
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    ((n ^ (n >> 16)) & 65535) as f32 / 65535.0 - 0.5
}

/// Grid patches may have sloping sides; lighting/noise is attached to the material.
pub fn surface(
    p: &Painter,
    t: Transform,
    top: f32,
    bottom: f32,
    left: impl Fn(f32) -> f32,
    right: impl Fn(f32) -> f32,
    base: Color32,
    grain: f32,
    step: f32,
    light: impl Fn(f32, f32) -> f32,
) {
    let rows = ((bottom - top) / step).ceil().max(1.0) as usize;
    let cols = ((right(top) - left(top)).max(right(bottom) - left(bottom)) / step)
        .ceil()
        .max(1.0) as usize;
    let mut mesh = Mesh::default();
    for row in 0..=rows {
        let v = row as f32 / rows as f32;
        let y = top + (bottom - top) * v;
        for col in 0..=cols {
            let u = col as f32 / cols as f32;
            mesh.vertices.push(Vertex {
                pos: t.pos(left(y) + (right(y) - left(y)) * u, y),
                uv: WHITE_UV,
                color: shade(
                    base,
                    light(u, v) + noise(col as u32, row as u32, 47) * grain,
                ),
            });
        }
    }
    for row in 0..rows {
        for col in 0..cols {
            let i = (row * (cols + 1) + col) as u32;
            let stride = (cols + 1) as u32;
            mesh.indices.extend_from_slice(&[
                i,
                i + 1,
                i + stride,
                i + 1,
                i + stride + 1,
                i + stride,
            ]);
        }
    }
    p.add(Shape::mesh(mesh));
}

/// Molded outline: narrow shoulders, slightly bowed sides, broad lower corners.
pub fn outline(p: &Painter, t: Transform, inset: f32, base: Color32) {
    let controls = [
        (33.0 + inset, 3.0 + inset),
        (297.0 - inset, 3.0 + inset),
        (310.0 - inset, 12.0 + inset),
        (326.0 - inset, 605.0 - inset),
        (317.0 - inset, 617.0 - inset),
        (13.0 + inset, 617.0 - inset),
        (4.0 + inset, 605.0 - inset),
        (20.0 + inset, 12.0 + inset),
    ];
    let points: Vec<_> = controls.iter().map(|&(x, y)| t.pos(x, y)).collect();
    let mut rounded = Vec::new();
    for i in 0..points.len() {
        let current = points[i];
        let prev = points[(i + points.len() - 1) % points.len()];
        let next = points[(i + 1) % points.len()];
        let a = current + (prev - current).normalized() * t.s(7.4);
        let b = current + (next - current).normalized() * t.s(7.4);
        rounded.push(a);
        for j in 1..=8 {
            let f = j as f32 / 8.0;
            rounded.push(a.lerp(current, f).lerp(current.lerp(b, f), f));
        }
    }
    p.add(Shape::convex_polygon(rounded.clone(), base, Stroke::NONE));
    let mut mesh = Mesh::default();
    mesh.vertices.push(Vertex {
        pos: t.pos(165.0, 310.0),
        uv: WHITE_UV,
        color: shade(base, 1.0),
    });
    for point in &rounded {
        let x = (point.x - t.origin.x) / t.scale;
        let y = (point.y - t.origin.y) / t.scale;
        let metal = if (11.0..16.5).contains(&inset) {
            1.6
        } else {
            0.7
        };
        let amount =
            (16.0 * (1.0 - x / 330.0) - 12.0 * x / 330.0 + 7.0 * (1.0 - y / 620.0)) * metal;
        mesh.vertices.push(Vertex {
            pos: *point,
            uv: WHITE_UV,
            color: shade(base, amount),
        });
    }
    for i in 0..rounded.len() as u32 {
        mesh.indices
            .extend_from_slice(&[0, i + 1, (i + 1) % rounded.len() as u32 + 1]);
    }
    p.add(Shape::mesh(mesh));
}

pub fn panel_left(y: f32) -> f32 {
    if y < 50.0 {
        return 49.0 - (y - 22.0) * 8.2 / 28.0;
    }
    42.0 - 14.0 * ((y - 16.0) / 586.0).clamp(0.0, 1.0)
}
pub fn panel_right(y: f32) -> f32 {
    330.0 - panel_left(y)
}
