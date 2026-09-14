//! Analytic surface lighting and deterministic vector microtexture, never a bitmap.
use super::geometry::Transform;
use eframe::egui::{
    epaint::{Mesh, Vertex, WHITE_UV},
    Color32, Painter, Shape, Stroke,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, OnceLock},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Material {
    Panel,
    CardRail,
    Glass,
    DisplayLip,
    SwitchRail,
    SwitchFoot,
    Slider,
    Nose,
    NoseInset,
    KeyFace,
    KeyFront,
}
#[derive(Hash, PartialEq, Eq)]
enum CacheKey {
    Surface(Material, [u32; 10], Color32),
    Outline(u32, Color32),
}
pub struct MaterialMesh {
    mesh: Mesh,
    boundaries: Vec<Vec<usize>>,
}
thread_local! {
    static MATERIALS: RefCell<HashMap<CacheKey, Arc<MaterialMesh>>> = RefCell::new(HashMap::new());
}
fn cached(key: CacheKey, build: impl FnOnce() -> MaterialMesh) -> Arc<MaterialMesh> {
    MATERIALS.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(mesh) = cache.get(&key) {
            return Arc::clone(mesh);
        }
        // The fixed panel needs fewer than 40 variants, including both switch states.
        if cache.len() >= 64 {
            cache.clear();
        }
        let mesh = Arc::new(build());
        cache.insert(key, Arc::clone(&mesh));
        mesh
    })
}
pub fn paint_material(p: &Painter, t: Transform, material: &MaterialMesh, aa: bool) {
    let mut mesh = material.mesh.clone();
    for v in &mut mesh.vertices {
        v.pos = (v.pos.to_vec2() * t.scale).to_pos2();
    }
    if aa {
        for boundary in &material.boundaries {
            feather_boundary(&mut mesh, boundary, p.ctx().pixels_per_point());
        }
    }
    for v in &mut mesh.vertices {
        v.pos += t.origin.to_vec2();
    }
    p.add(Shape::mesh(mesh));
}

/// Shared, broad illumination from the upper left. Values are tiny sRGB deltas.
pub fn soft_light(u: f32, v: f32) -> f32 {
    4.0 * (1.0 - u) + 3.0 * (1.0 - v) - 3.5
}

/// Only stationary straight seams are aligned; artwork and layout stay continuous.
pub fn aligned_horizontal(
    p: &Painter,
    t: Transform,
    x0: f32,
    x1: f32,
    y: f32,
    width: f32,
    color: Color32,
) {
    let ppp = p.ctx().pixels_per_point();
    let physical_width = t.s(width) * ppp;
    let pixels = physical_width.round().max(1.0);
    let phase = if pixels as u32 % 2 == 0 { 0.0 } else { 0.5 };
    let py = ((t.pos(0.0, y).y * ppp - phase).round() + phase) / ppp;
    // Preserve subpixel line coverage instead of fattening all strokes to a pixel.
    p.line_segment(
        [
            eframe::egui::pos2(t.pos(x0, y).x, py),
            eframe::egui::pos2(t.pos(x1, y).x, py),
        ],
        Stroke::new(t.s(width), color),
    );
}

pub fn shade(base: Color32, amount: f32) -> Color32 {
    let channel = |v: u8| (v as f32 + amount).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(channel(base.r()), channel(base.g()), channel(base.b()))
}

fn hash_noise(x: u32, y: u32, seed: u32) -> f32 {
    let mut n = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        ^ seed;
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    ((n ^ (n >> 16)) & 65535) as f32 / 65535.0 - 0.5
}
fn noise(x: u32, y: u32) -> f32 {
    static GRAIN: OnceLock<Vec<f32>> = OnceLock::new();
    let tile = GRAIN.get_or_init(|| {
        (0..64 * 64)
            .map(|i| {
                let (x, y) = (i % 64, i / 64);
                0.5 * hash_noise(x, y, 47)
                    + 0.25 * hash_noise(x + 1, y, 47)
                    + 0.25 * hash_noise(x, y + 1, 47)
            })
            .collect()
    });
    tile[((y % 64) * 64 + x % 64) as usize]
}

/// Grid patches may have sloping sides; lighting/noise is attached to the material.
pub fn surface(
    p: &Painter,
    t: Transform,
    material: Material,
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
        p, t, material, top, bottom, left, right, base, grain, step, light, false,
    );
}

pub fn antialiased_surface(
    p: &Painter,
    t: Transform,
    material: Material,
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
        p, t, material, top, bottom, left, right, base, grain, step, light, true,
    );
}

fn surface_impl(
    p: &Painter,
    t: Transform,
    material: Material,
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
    // Each material id owns one fixed lighting/edge profile. Variable dimensions,
    // position, color and grain are part of the key; scale/DPI and travel are not.
    let mid = (top + bottom) * 0.5;
    let key = CacheKey::Surface(
        material,
        [
            top,
            bottom,
            left(top),
            right(top),
            left(bottom),
            right(bottom),
            left(mid),
            right(mid),
            grain,
            step,
        ]
        .map(f32::to_bits),
        base,
    );
    let cached = cached(key, || {
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
                    pos: eframe::egui::pos2(left(y) + (right(y) - left(y)) * u, y),
                    uv: WHITE_UV,
                    color: shade(base, light(u, v) + noise(col as u32, row as u32) * grain),
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
        MaterialMesh {
            mesh,
            boundaries: vec![boundary],
        }
    });
    paint_material(p, t, &cached, edge_aa);
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
    let material = cached(CacheKey::Outline(inset.to_bits(), base), || {
        let rounded = subdivided(
            &outline_points(inset)
                .into_iter()
                .map(|(x, y)| eframe::egui::pos2(x, y))
                .collect::<Vec<_>>(),
        );
        let mut mesh = Mesh::default();
        mesh.vertices.push(Vertex {
            pos: eframe::egui::pos2(165.0, 310.0),
            uv: WHITE_UV,
            color: shade(base, 1.0),
        });
        for fraction in [0.96, 1.0] {
            for (i, point) in rounded.iter().enumerate() {
                let point = eframe::egui::pos2(165.0, 310.0).lerp(*point, fraction);
                // The lower return belongs to the molded shell itself, so its
                // shadow flows through the existing case rings without a tile.
                let return_shade = if inset < 10.0 {
                    6.0 * ((point.y - 605.0) / 12.0).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let amount = soft_light(point.x / 330.0, point.y / 620.0) * 1.4 - return_shade
                    + 2.5 * (1.0 - fraction)
                    - 2.0 * fraction.powi(8)
                    + 1.6 * noise(i as u32, (fraction * 100.0) as u32);
                mesh.vertices.push(Vertex {
                    pos: point,
                    uv: WHITE_UV,
                    color: shade(base, amount),
                });
            }
        }
        for i in 0..rounded.len() as u32 {
            mesh.indices
                .extend_from_slice(&[0, i + 1, (i + 1) % rounded.len() as u32 + 1]);
        }
        let n = rounded.len() as u32;
        for ring in 0..1 {
            for i in 0..n {
                let a = 1 + ring * n + i;
                let b = 1 + ring * n + (i + 1) % n;
                mesh.indices
                    .extend_from_slice(&[a, b, a + n, b, b + n, a + n]);
            }
        }
        MaterialMesh {
            mesh,
            boundaries: vec![(1 + n..1 + 2 * n).map(|i| i as usize).collect()],
        }
    });
    paint_material(p, t, &material, true);
}

fn subdivided(points: &[eframe::egui::Pos2]) -> Vec<eframe::egui::Pos2> {
    let mut out = Vec::new();
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        let steps = (a.distance(b) / 4.0).ceil().max(1.0) as usize;
        for j in 0..steps {
            out.push(a.lerp(b, j as f32 / steps as f32));
        }
    }
    out
}

/// Satin metal follows the existing centerline, narrowing over the lower fold.
pub fn metal_rim(points: &[eframe::egui::Pos2], silver: Color32) -> MaterialMesh {
    let points = subdivided(points);
    let mut mesh = Mesh::default();
    let n = points.len();
    for (i, point) in points.iter().enumerate() {
        let a = (*point - points[(i + n - 1) % n]).normalized();
        let b = (points[(i + 1) % n] - *point).normalized();
        let na = eframe::egui::vec2(a.y, -a.x);
        let nb = eframe::egui::vec2(b.y, -b.x);
        let normal = (na + nb).normalized();
        let miter = normal / normal.dot(na).max(0.5);
        let fold = ((point.y - 585.5) / 19.5).clamp(0.0, 1.0);
        let width = 1.0 - 0.36 * fold;
        for (band, (offset, delta)) in [
            (-2.2, -128.0),
            (-1.5, -80.0),
            (-0.7, -29.0),
            (0.0, -15.0),
            (0.7, -23.0),
            (1.5, -54.0),
            (2.2, -88.0),
        ]
        .into_iter()
        .enumerate()
        {
            let direction = normal.dot(eframe::egui::vec2(-0.6, -0.8)) * 2.5;
            let brush = noise((i / 5) as u32, band as u32) * 1.4 + noise(i as u32, 0) * 0.4;
            mesh.vertices.push(Vertex {
                pos: *point + miter * offset * width,
                uv: WHITE_UV,
                color: shade(
                    silver,
                    delta + direction + soft_light(point.x / 330.0, point.y / 620.0) + brush
                        - 19.0 * fold,
                ),
            });
        }
    }
    for i in 0..n {
        for band in 0..6 {
            let a = (i * 7 + band) as u32;
            let b = (((i + 1) % n) * 7 + band) as u32;
            mesh.indices
                .extend_from_slice(&[a, b, a + 1, b, b + 1, a + 1]);
        }
    }
    MaterialMesh {
        mesh,
        boundaries: vec![
            (0..n).rev().map(|i| i * 7).collect(),
            (0..n).map(|i| i * 7 + 6).collect(),
        ],
    }
}

pub(crate) fn outline_points(inset: f32) -> Vec<(f32, f32)> {
    use super::geometry::{CASE_BOTTOM, CASE_SIDE_EXPANSION, CASE_TOP};
    let top = CASE_TOP + inset;
    let bottom = CASE_BOTTOM - inset;
    let right = 318.0 + CASE_SIDE_EXPANSION - inset;
    let radius = 9.0;
    // Flatten only the molded lower corner's vertical roll. Overall bounds,
    // long-side width and the metal's upper corners keep their existing anchors.
    let lower_radius = if inset < 10.0 { 5.0 } else { radius };
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
            top + radius + (bottom - top - radius - lower_radius) * f,
        ));
    }
    // The shorter lower roll keeps equal end widths without a rounded UI corner.
    for i in 0..=12 {
        let f = i as f32 / 12.0;
        points.push((
            right - radius * f * f,
            bottom - lower_radius * (1.0 - f) * (1.0 - f),
        ));
    }
    let right_half = points.clone();
    points.extend(right_half.into_iter().rev().map(|(x, y)| (330.0 - x, y)));
    points
}

pub fn panel_left(y: f32) -> f32 {
    let f = ((y - 29.0) / 562.0).clamp(0.0, 1.0);
    29.0 - super::geometry::CASE_SIDE_EXPANSION - 7.0 * (std::f32::consts::PI * f).sin()
}
pub fn panel_right(y: f32) -> f32 {
    330.0 - panel_left(y)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_lighting_is_reused_across_scale_and_separates_color_variants() {
        let ctx = eframe::egui::Context::default();
        let evaluations = std::cell::Cell::new(0);
        let render = |scale, color| {
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                antialiased_surface(
                    &ctx.layer_painter(eframe::egui::LayerId::background()),
                    Transform {
                        origin: eframe::egui::Pos2::ZERO,
                        scale,
                    },
                    Material::KeyFace,
                    0.0,
                    20.0,
                    |_| 0.0,
                    |_| 36.0,
                    color,
                    0.9,
                    1.2,
                    |u, v| {
                        evaluations.set(evaluations.get() + 1);
                        soft_light(u, v)
                    },
                );
            });
        };
        render(1.0, Color32::GRAY);
        let first = evaluations.get();
        assert!(first > 0);
        render(2.0, Color32::GRAY);
        assert_eq!(first, evaluations.get());
        render(2.0, Color32::DARK_GRAY);
        assert!(evaluations.get() > first);
        let both = evaluations.get();
        render(1.37, Color32::GRAY);
        assert_eq!(both, evaluations.get());
    }
    #[test]
    fn actual_case_outline_keeps_the_published_plan_ratio() {
        let bounds =
            outline_points(0.0)
                .into_iter()
                .fold(eframe::egui::Rect::NOTHING, |mut r, (x, y)| {
                    r.extend_with(eframe::egui::pos2(x, y));
                    r
                });
        assert!(
            (bounds.height() / bounds.width()
                - super::super::geometry::CASE_LENGTH_MM / super::super::geometry::CASE_WIDTH_MM)
                .abs()
                < 0.00001
        );
    }
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
