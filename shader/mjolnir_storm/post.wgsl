// Mjolnir storm: one post pass driven by the mod's native half through frame.mods[0]:
//   x  flash, 0..1 (a white-blue wash that fades after each strike)
//   y  cinematic, 0..1 (eases in and out while Thor's wrath plays)
//   z  impact cut: 0 none, 1 white with black silhouettes, 2 negative, 3 white with silhouettes
//   w  release, 1..0 over 0.4 s after the cuts: zoom blur, colour split and glow
// With all four at 0 the picture is returned unchanged.

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

fn post(input: PostInput) -> vec4<f32> {
    let m = frame.mods[0];
    let flash = clamp(m.x, 0.0, 1.0);
    let cine = clamp(m.y, 0.0, 1.0);
    let cut = i32(m.z + 0.5);
    let release = clamp(m.w, 0.0, 1.0);

    let uv = input.uv;
    let d = uv - vec2<f32>(0.5, 0.5);
    let dist = length(d);

    // Hard cuts: black silhouettes and converging lines on white, then a negative, then white again.
    if (cut == 1 || cut == 3) {
        let solid = step(0.00001, input.depth);
        let rays = step(0.86, fract(sin(floor(atan2(d.y, d.x) * 48.0 + 0.5) * 12.9898) * 43758.5453));
        let lines = rays * smoothstep(0.12, 0.45, dist);
        let ink = max(solid, lines);
        return vec4<f32>(vec3<f32>(1.0 - ink), input.color.a);
    }
    if (cut == 2) {
        let negative = vec3<f32>(1.0) - input.color.rgb;
        let l = luma(negative);
        let contrast = clamp((l - 0.5) * 2.2 + 0.5, 0.0, 1.0);
        return vec4<f32>(vec3<f32>(contrast) * vec3<f32>(0.8, 0.92, 1.0), input.color.a);
    }

    // Zoom blur with a colour split: strong as the cuts let go, a hint while the strike is on.
    let strength = max(release * 0.09, cine * 0.012 + flash * 0.02);
    var acc = vec3<f32>(0.0);
    for (var i = 0; i < 6; i = i + 1) {
        let k = f32(i) / 5.0;
        let off = d * (strength * k);
        let r = textureSampleLevel(previous, linear_sampler, uv - off * 1.15, 0.0).r;
        let g = textureSampleLevel(previous, linear_sampler, uv - off, 0.0).g;
        let b = textureSampleLevel(previous, linear_sampler, uv - off * 0.85, 0.0).b;
        acc = acc + vec3<f32>(r, g, b);
    }
    var color = acc / 6.0;

    // Glow: whatever is bright blooms, most of all as the cuts let go.
    let bright = max(color - vec3<f32>(0.55), vec3<f32>(0.0));
    color = color + bright * (0.4 + 2.5 * release + 1.5 * flash);

    // A cold storm grade and a vignette while the cinematic runs.
    let l = luma(color);
    color = mix(color, vec3<f32>(l) * vec3<f32>(0.78, 0.92, 1.25), 0.4 * cine);
    color = color * (1.0 - cine * 0.55 * smoothstep(0.3, 0.85, dist * 1.4));

    return vec4<f32>(color, input.color.a);
}
