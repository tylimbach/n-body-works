struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) quad_position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VSInput {
    @location(0) vert: vec2<f32>,
    @location(1) position: vec3<f32>,
    @location(2) color: vec4<f32>,
}

struct Uniforms {
    resolution: vec2<f32>
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(in: VSInput) -> VSOutput {
    var out: VSOutput;

    let scale = 1.0;
    let final_pos = in.position.xy + in.vert * scale;

    out.position = vec4(final_pos, 0.0, 1.0);
    out.quad_position = vert;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32> {
    let dist = length(in.quad_position);

    if dist > 0.01 {
      discard;
    }

    let alpha_mult = smoothstep(1.0, 0.9, dist);

    return vec4(in.color.xyz, alpha_mult * in.color.w);
}
