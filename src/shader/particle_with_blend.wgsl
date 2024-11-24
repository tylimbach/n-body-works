struct VSOutput {
    @builtin(position) frag_coord: vec4<f32>,
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
@group(0) @binding(1) var prev_frame: texture_2d<f32>;
@group(0) @binding(2) var prev_sampler: sampler;

@vertex
fn vs_main(in: VSInput) -> VSOutput {
    var out: VSOutput;

    let scale = 1.0;
    let final_pos = in.position.xy + in.vert * scale;

    out.frag_coord = vec4(final_pos, 0.0, 1.0);
    out.quad_position = in.position.xy;
    out.color = in.position * 0.5 + 0.5;

    return out;
}

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32> {
    let uv = in.frag_coord.xy / global.resolution;
    let prev_color = textureSample(prev_frame, prev_sampler, uv)

    let dist = length(in.quad_position);

    if dist > 0.01 {
         discard;
    }

    let alpha_mult = smoothstep(1.0, 0.5, dist);

    return vec4(in.color.xy, 0.5, alpha_mult);
}
