//! Portable outline lettering with per-legend optical metrics. No system-font lookup.
//! Contours are derived from OFL Arimo; see tools/typography for provenance.
use super::lettering_data::glyph;
use eframe::egui::{
    epaint::{Mesh, Vertex, WHITE_UV},
    Align2, Color32, Painter, Pos2, Shape, Vec2,
};

pub fn text(
    p: &Painter,
    pos: Pos2,
    height: f32,
    color: Color32,
    value: &str,
    align: Align2,
    weight: u16,
) {
    let cap = height * 0.78;
    let tracking = 0.028;
    let width = value
        .chars()
        .map(|c| glyph(c, weight).advance + tracking)
        .sum::<f32>()
        * cap;
    let origin = align.anchor_size(pos, Vec2::new(width, cap)).min;
    let mut mesh = Mesh::default();
    let mut cursor = 0.0;
    for c in value.chars() {
        let g = glyph(c, weight);
        let offset = mesh.vertices.len() as u32;
        mesh.indices.extend(g.indices.iter().map(|i| i + offset));
        for &[x, y] in &g.vertices {
            mesh.vertices.push(Vertex {
                pos: origin + Vec2::new((cursor + x) * cap, y * cap),
                uv: WHITE_UV,
                color,
            });
        }
        // A subpixel coverage fringe around the true contours gives smooth
        // boundaries at any scale, including the counters inside letters.
        for path in &g.contours {
            let points: Vec<_> = path
                .iter()
                .map(|&[x, y]| origin + Vec2::new((cursor + x) * cap, y * cap))
                .collect();
            let start = mesh.vertices.len() as u32;
            for i in 0..points.len() {
                let a = (points[i] - points[(i + points.len() - 1) % points.len()]).normalized();
                let b = (points[(i + 1) % points.len()] - points[i]).normalized();
                let na = Vec2::new(a.y, -a.x);
                let nb = Vec2::new(b.y, -b.x);
                let normal = (na + nb).normalized();
                let outside = points[i]
                    + normal * (0.55 / p.ctx().pixels_per_point() / normal.dot(na).max(0.25));
                mesh.vertices.push(Vertex {
                    pos: points[i],
                    uv: WHITE_UV,
                    color,
                });
                mesh.vertices.push(Vertex {
                    pos: outside,
                    uv: WHITE_UV,
                    color: Color32::TRANSPARENT,
                });
            }
            for i in 0..points.len() as u32 {
                let a = start + i * 2;
                let b = start + ((i + 1) % points.len() as u32) * 2;
                mesh.indices
                    .extend_from_slice(&[a, b, a + 1, b, b + 1, a + 1]);
            }
        }
        cursor += g.advance + tracking;
    }
    p.add(Shape::mesh(mesh));
}
