struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    let p = positions[index];
    var out: VertexOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    out.uv = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
    return out;
}

fn hash21(p: vec2<f32>) -> f32 {
    let q = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    let r = q + vec3<f32>(dot(q, q.yzx + vec3<f32>(33.33)));
    return fract((r.x + r.y) * r.z);
}

fn box_mask(p: vec2<f32>, lo: vec2<f32>, hi: vec2<f32>, feather: f32) -> f32 {
    let a = smoothstep(lo, lo + vec2<f32>(feather), p);
    let b = vec2<f32>(1.0) - smoothstep(hi - vec2<f32>(feather), hi, p);
    return a.x * a.y * b.x * b.y;
}

fn gaussian(x: f32, width: f32) -> f32 {
    let q = x / width;
    return exp(-q * q);
}

fn rounded_rect_sdf(p: vec2<f32>, center: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p - center) - (half_size - vec2<f32>(radius));
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

// Signed photographic contribution for one physical key. Positive values are
// reflected light on the slightly convex cap; negative values are socket/contact
// occlusion and the lower-right molded falloff. Coordinates are the 330x620
// measured design space used by the vector panel.
fn key_exposure(p: vec2<f32>, center: vec2<f32>, size: vec2<f32>) -> f32 {
    let half_size = size * 0.5;
    let sd = rounded_rect_sdf(p, center, half_size, 2.4);
    let inside = 1.0 - smoothstep(-0.55, 0.85, sd);
    let outside = step(0.0, sd) * (1.0 - smoothstep(0.0, 3.2, sd));
    let local = (p - center) / half_size;
    let radial = clamp(1.0 - length(local * vec2<f32>(0.80, 1.02)), 0.0, 1.0);
    let upper_left = clamp((1.0 - local.x) * 0.5 + (1.0 - local.y) * 0.5, 0.0, 2.0);
    let lower_edge = smoothstep(0.42, 1.02, local.y);
    let right_edge = smoothstep(0.55, 1.02, local.x);
    let individual = (hash21(center * 0.173) - 0.5) * 0.010;
    let convex = inside * (0.020 + radial * 0.024 + upper_left * 0.007 + individual);
    let molded_falloff = inside * (lower_edge * 0.035 + right_edge * 0.016);
    let contact_ao = outside * 0.070;
    return convex - molded_falloff - contact_ao;
}

fn keyboard_exposure(p: vec2<f32>) -> f32 {
    var e = 0.0;

    // A-E
    e += key_exposure(p, vec2<f32>(54.5, 190.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(110.5, 190.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(166.0, 190.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(221.5, 190.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(277.0, 190.0), vec2<f32>(36.0, 30.0));

    // Sigma/GTO/DSP/(i)/SST
    e += key_exposure(p, vec2<f32>(54.5, 243.5), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(110.5, 243.5), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(166.0, 243.5), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(221.5, 243.5), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(277.0, 243.5), vec2<f32>(36.0, 30.0));

    // f/g/STO/RCL/h
    e += key_exposure(p, vec2<f32>(54.5, 296.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(110.5, 296.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(166.0, 296.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(221.5, 296.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(277.0, 296.0), vec2<f32>(36.0, 30.0));

    // ENTER / CHS / EEX / CLx
    e += key_exposure(p, vec2<f32>(84.5, 348.5), vec2<f32>(92.0, 31.0));
    e += key_exposure(p, vec2<f32>(166.0, 348.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(221.5, 348.0), vec2<f32>(36.0, 30.0));
    e += key_exposure(p, vec2<f32>(277.0, 348.0), vec2<f32>(36.0, 30.0));

    // Numeric/operator rows
    e += key_exposure(p, vec2<f32>(53.5, 398.75), vec2<f32>(28.0, 29.5));
    e += key_exposure(p, vec2<f32>(118.5, 398.75), vec2<f32>(42.0, 29.5));
    e += key_exposure(p, vec2<f32>(195.5, 398.75), vec2<f32>(42.0, 29.5));
    e += key_exposure(p, vec2<f32>(274.0, 398.75), vec2<f32>(42.0, 29.5));

    e += key_exposure(p, vec2<f32>(53.5, 450.25), vec2<f32>(28.0, 29.5));
    e += key_exposure(p, vec2<f32>(118.5, 450.25), vec2<f32>(42.0, 29.5));
    e += key_exposure(p, vec2<f32>(195.5, 450.25), vec2<f32>(42.0, 29.5));
    e += key_exposure(p, vec2<f32>(274.0, 450.25), vec2<f32>(42.0, 29.5));

    e += key_exposure(p, vec2<f32>(53.5, 501.25), vec2<f32>(28.0, 29.5));
    e += key_exposure(p, vec2<f32>(118.5, 501.25), vec2<f32>(42.0, 29.5));
    e += key_exposure(p, vec2<f32>(195.5, 501.25), vec2<f32>(42.0, 29.5));
    e += key_exposure(p, vec2<f32>(274.0, 501.25), vec2<f32>(42.0, 29.5));

    e += key_exposure(p, vec2<f32>(53.5, 551.25), vec2<f32>(28.0, 29.5));
    e += key_exposure(p, vec2<f32>(118.5, 551.25), vec2<f32>(42.0, 29.5));
    e += key_exposure(p, vec2<f32>(195.5, 551.25), vec2<f32>(42.0, 29.5));
    e += key_exposure(p, vec2<f32>(274.0, 551.25), vec2<f32>(42.0, 29.5));

    return e;
}

@fragment
fn fx_main(in: VertexOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let design = uv * vec2<f32>(330.0, 620.0);

    let nearest_edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    let body = smoothstep(0.003, 0.018, nearest_edge);

    // Deliberately visible photographic lighting for this pass: one broad key
    // light, a weak warm desk bounce, and lens/camera falloff.
    let key_light = max(0.0, (1.0 - uv.x) * 0.028 + (1.0 - uv.y) * 0.020 - 0.020);
    let desk_bounce = smoothstep(0.58, 0.98, uv.y) * smoothstep(0.45, 0.95, uv.x) * 0.012;
    let d = (uv - vec2<f32>(0.47, 0.45)) * vec2<f32>(1.0, 0.58);
    let vignette = smoothstep(0.28, 0.64, length(d)) * 0.052;

    let px = floor(uv * vec2<f32>(660.0, 1240.0));
    let fine_grain = (hash21(px) - 0.5) * 0.015;
    let medium_grain = (hash21(floor(px / 7.0)) - 0.5) * 0.009;
    let low_grain = (hash21(floor(px / 29.0)) - 0.5) * 0.012;

    // Stronger satin/brushed response on the rolled aluminium rim.
    let side_rim = max(
        gaussian(abs(uv.x - 0.0415), 0.0080),
        gaussian(abs(uv.x - 0.9585), 0.0080)
    ) * smoothstep(0.025, 0.08, uv.y) * (1.0 - smoothstep(0.90, 0.985, uv.y));
    let top_rim = gaussian(abs(uv.y - 0.0225), 0.0070) * smoothstep(0.05, 0.20, uv.x) * (1.0 - smoothstep(0.80, 0.95, uv.x));
    let brush = 0.58 + 0.42 * sin(uv.y * 1180.0 + uv.x * 37.0);
    let long_brush = 0.80 + 0.20 * sin(uv.y * 89.0 + 0.7);
    let rim_glint = (side_rim * brush * long_brush + top_rim) * 0.085;

    // Dark display filter with a much more legible Fresnel edge and asymmetric
    // room-light reflection. This remains an optical overlay: LED geometry stays sharp.
    let glass_lo = vec2<f32>(24.0 / 330.0, 22.5 / 620.0);
    let glass_hi = vec2<f32>(306.0 / 330.0, 79.5 / 620.0);
    let glass = box_mask(uv, glass_lo, glass_hi, 0.004);
    let gc = (glass_lo + glass_hi) * 0.5;
    let gh = (glass_hi - glass_lo) * 0.5;
    let gq = abs((uv - gc) / gh);
    let glass_edge = clamp(max(gq.x, gq.y), 0.0, 1.0);
    let fresnel = glass * pow(glass_edge, 5.0) * 0.060;
    let reflection_y = glass_lo.y + 0.018 + (uv.x - 0.45) * 0.014;
    let reflection_a = glass * gaussian(uv.y - reflection_y, 0.010) * (0.30 + 0.70 * (1.0 - uv.x)) * 0.100;
    let reflection_b = glass * gaussian(uv.y - (glass_lo.y + 0.052), 0.020) * smoothstep(0.10, 0.45, uv.x) * (1.0 - smoothstep(0.55, 0.90, uv.x)) * 0.025;
    let glass_center_dark = glass * (1.0 - pow(glass_edge, 3.0)) * 0.018;

    // GPU-only depth cue absent from pass 1: every physical key receives a
    // convex highlight and a narrow contact shadow around its socket.
    let keys = keyboard_exposure(design);

    // A restrained broad plastic response over the lower molded case/nose.
    let lower_case = smoothstep(0.84, 0.95, uv.y) * (1.0 - smoothstep(0.985, 1.0, uv.y));
    let case_roll = lower_case * ((1.0 - uv.x) * 0.018 - uv.y * 0.004);

    let signed_exposure = (
        key_light
        + desk_bounce
        + rim_glint
        + fresnel
        + reflection_a
        + reflection_b
        + keys
        + case_roll
        + fine_grain
        + medium_grain
        + low_grain
        - vignette
        - glass_center_dark
    ) * body;

    if signed_exposure >= 0.0 {
        let alpha = clamp(signed_exposure, 0.0, 0.16);
        // Warm reflected key light; stronger than pass 1 so the experiment is
        // visually obvious, but still below a stylized bloom/high-gloss look.
        return vec4<f32>(1.0, 0.955, 0.86, alpha);
    }

    let alpha = clamp(-signed_exposure, 0.0, 0.12);
    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}

@group(0) @binding(0)
var fx_texture: texture_2d<f32>;
@group(0) @binding(1)
var fx_sampler: sampler;

@fragment
fn composite_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(fx_texture, fx_sampler, in.uv);
}
