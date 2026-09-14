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
    // Keep uv.y = 0 at the physical top of the calculator.
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

@fragment
fn fx_main(in: VertexOut) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // The design canvas is almost exactly the physical case outline; keep the
    // effect off the few-pixel exterior margin and feather it at the silhouette.
    let nearest_edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    let body = smoothstep(0.003, 0.018, nearest_edge);

    // Broad photographic key light from the upper left plus weak lens/camera
    // falloff. These are intentionally tiny so printed legends stay crisp.
    let key_light = max(0.0, (1.0 - uv.x) * 0.010 + (1.0 - uv.y) * 0.006 - 0.007);
    let d = (uv - vec2<f32>(0.48, 0.46)) * vec2<f32>(1.0, 0.58);
    let vignette = smoothstep(0.30, 0.64, length(d)) * 0.026;

    // Deterministic sensor/material variation. This is generated in the
    // offscreen texture, so it does not shimmer between frames.
    let px = floor(uv * vec2<f32>(660.0, 1240.0));
    let fine_grain = (hash21(px) - 0.5) * 0.010;
    let low_grain = (hash21(floor(px / 23.0)) - 0.5) * 0.006;

    // Satin rolled-metal rim. The weak high-frequency modulation reads as
    // brushed metal without turning into visible stripes.
    let side_rim = max(
        gaussian(abs(uv.x - 0.0415), 0.0065),
        gaussian(abs(uv.x - 0.9585), 0.0065)
    ) * smoothstep(0.025, 0.08, uv.y) * (1.0 - smoothstep(0.90, 0.985, uv.y));
    let top_rim = gaussian(abs(uv.y - 0.0225), 0.0060) * smoothstep(0.05, 0.20, uv.x) * (1.0 - smoothstep(0.80, 0.95, uv.x));
    let brush = 0.72 + 0.28 * sin(uv.y * 1220.0 + uv.x * 31.0);
    let rim_glint = (side_rim * brush + top_rim) * 0.026;

    // Real HP-67 display glass occupies x=24..306, y=22.5..79.5 on a
    // 330x620 design canvas. Add edge Fresnel and a soft asymmetric room-light
    // reflection while leaving the LED geometry itself untouched.
    let glass_lo = vec2<f32>(24.0 / 330.0, 22.5 / 620.0);
    let glass_hi = vec2<f32>(306.0 / 330.0, 79.5 / 620.0);
    let glass = box_mask(uv, glass_lo, glass_hi, 0.004);
    let gc = (glass_lo + glass_hi) * 0.5;
    let gh = (glass_hi - glass_lo) * 0.5;
    let gq = abs((uv - gc) / gh);
    let fresnel = glass * pow(clamp(max(gq.x, gq.y), 0.0, 1.0), 7.0) * 0.022;
    let reflection_y = glass_lo.y + 0.018 + (uv.x - 0.5) * 0.010;
    let glass_reflection = glass * gaussian(uv.y - reflection_y, 0.009) * (0.35 + 0.65 * (1.0 - uv.x)) * 0.040;

    let signed_exposure = (key_light + rim_glint + fresnel + glass_reflection + fine_grain + low_grain - vignette) * body;

    if signed_exposure >= 0.0 {
        let alpha = clamp(signed_exposure, 0.0, 0.055);
        // Slightly warm incident light, as in a photographed desk/lab scene.
        return vec4<f32>(1.0, 0.965, 0.90, alpha);
    }

    let alpha = clamp(-signed_exposure, 0.0, 0.045);
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
