struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) quad_position: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VSInput {
    @location(0) vert: vec2<f32>,
    @location(1) position: vec3<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(in: VSInput) -> VSOutput {
    var out: VSOutput;

    let scale = 1.0;
    let final_pos = in.position.xy + in.vert * scale;

    out.position = vec4(final_pos, 0.0, 1.0);
    out.quad_position = in.vert;
    out.color = in.position * 0.5 + 0.5;

    return out;
}

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32> {
    let dist = length(in.quad_position);
    let alpha_mult = smoothstep(0.003, 0.001, dist);
    return vec4(in.color.xy, 0.5, alpha_mult);
}
