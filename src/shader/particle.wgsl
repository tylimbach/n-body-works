struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) quad_position: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VSInput {
    @location(0) vert: vec2<f32>,
    @location(1) position: vec3<f32>,
}

struct GlobalUniforms {
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> global: GlobalUniforms;
// @group(1) @binding(0) var texture_sample: texture_2d<f32>;

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
    // let uv = in.position.xy / global.resolution;
    // let prev_color = textureSample(previous)

    let dist = length(in.quad_position);

    if dist > 0.01 {
         discard;
    }

    let alpha_mult = smoothstep(1.0, 0.5, dist);

    return vec4(in.color.xy, 0.5, alpha_mult);
}
