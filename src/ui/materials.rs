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
    surface_impl(
        p, t, top, bottom, left, right, base, grain, step, light, false,
    );
}

pub fn antialiased_surface(
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
    surface_impl(
        p, t, top, bottom, left, right, base, grain, step, light, true,
    );
}

fn surface_impl(
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
    edge_aa: bool,
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
                pos: if edge_aa {
                    eframe::egui::Pos2::new(
                        (left(y) + (right(y) - left(y)) * u) * t.scale,
                        y * t.scale,
                    )
                } else {
                    t.pos(left(y) + (right(y) - left(y)) * u, y)
                },
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
    let stride = cols + 1;
    let boundary: Vec<_> = (0..=cols)
        .chain((1..=rows).map(|row| row * stride + cols))
        .chain((0..cols).rev().map(|col| rows * stride + col))
        .chain((1..rows).rev().map(|row| row * stride))
        .collect();
    if edge_aa {
        feather_boundary(&mut mesh, &boundary, p.ctx().pixels_per_point());
        for vertex in &mut mesh.vertices {
            vertex.pos += t.origin.to_vec2();
        }
    }
    p.add(Shape::mesh(mesh));
}

/// One physical-pixel coverage ramp on exposed surface edges. The LED renderer
/// does not use this mesh helper, so display geometry/coverage stays unchanged.
fn feather_boundary(mesh: &mut Mesh, boundary: &[usize], ppp: f32) {
    let start = mesh.vertices.len() as u32;
    for (i, &index) in boundary.iter().enumerate() {
        let v = mesh.vertices[index];
        let prev = mesh.vertices[boundary[(i + boundary.len() - 1) % boundary.len()]].pos;
        let next = mesh.vertices[boundary[(i + 1) % boundary.len()]].pos;
        let a = (v.pos - prev).normalized();
        let b = (next - v.pos).normalized();
        let na = eframe::egui::vec2(a.y, -a.x);
        let nb = eframe::egui::vec2(b.y, -b.x);
        let normal = (na + nb).normalized();
        mesh.vertices.push(Vertex {
            pos: v.pos + normal / (ppp * normal.dot(na).max(0.5)),
            color: Color32::TRANSPARENT,
            ..v
        });
    }
    for i in 0..boundary.len() {
        let j = (i + 1) % boundary.len();
        let a = boundary[i] as u32;
        let b = boundary[j] as u32;
        mesh.indices.extend_from_slice(&[
            a,
            b,
            start + i as u32,
            b,
            start + j as u32,
            start + i as u32,
        ]);
    }
}

/// Near-frontal outline: almost parallel ends, gently bowed long sides.
/// The oblique photographs exaggerate convergence; do not model that perspective
/// as a narrow top. All inset rims use the same envelope and corner construction.
pub fn outline(p: &Painter, t: Transform, inset: f32, base: Color32) {
    let rounded = outline_points(inset)
        .into_iter()
        .map(|(x, y)| t.pos(x, y))
        .collect::<Vec<_>>();
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

pub(crate) fn outline_points(inset: f32) -> Vec<(f32, f32)> {
    let top = 3.0 + inset;
    let bottom = 617.0 - inset;
    let right = 318.0 - inset;
    let radius = 9.0;
    let mut points = Vec::new();
    let mut corner = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        for i in 0..=12 {
            let f = i as f32 / 12.0;
            points.push((
                (1.0 - f) * (1.0 - f) * a.0 + 2.0 * f * (1.0 - f) * b.0 + f * f * c.0,
                (1.0 - f) * (1.0 - f) * a.1 + 2.0 * f * (1.0 - f) * b.1 + f * f * c.1,
            ));
        }
    };
    corner((right - radius, top), (right, top), (right, top + radius));
    for i in 1..=48 {
        let f = i as f32 / 48.0;
        points.push((
            right + 7.0 * (std::f32::consts::PI * f).sin(),
            top + radius + (bottom - top - 2.0 * radius) * f,
        ));
    }
    // Reflect the upper right corner downwards, then the complete right side
    // to the left. This keeps upper/lower widths equal instead of tapering.
    for i in 0..=12 {
        let f = i as f32 / 12.0;
        points.push((
            right - radius * f * f,
            bottom - radius * (1.0 - f) * (1.0 - f),
        ));
    }
    let right_half = points.clone();
    points.extend(right_half.into_iter().rev().map(|(x, y)| (330.0 - x, y)));
    points
}

pub fn panel_left(y: f32) -> f32 {
    let f = ((y - 29.0) / 562.0).clamp(0.0, 1.0);
    29.0 - 7.0 * (std::f32::consts::PI * f).sin()
}
pub fn panel_right(y: f32) -> f32 {
    330.0 - panel_left(y)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shoulders_do_not_converge_into_a_trapezoid() {
        assert!((panel_left(50.0) - panel_left(570.0)).abs() < 0.001);
        assert!(panel_left(310.0) < panel_left(50.0));
        for inset in [0.0, 13.0, 17.0] {
            let points = outline_points(inset);
            for &(x, y) in &points {
                assert!(x >= 0.0 && x <= 330.0 && y >= 0.0 && y <= 620.0);
            }
            let span = |y: f32| {
                let xs: Vec<_> = points
                    .iter()
                    .filter(|p| (p.1 - y).abs() < 0.001)
                    .map(|p| p.0)
                    .collect();
                xs.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                    - xs.iter().copied().fold(f32::INFINITY, f32::min)
            };
            assert!((span(3.0 + inset) - span(617.0 - inset)).abs() < 0.001);
        }
    }
}
