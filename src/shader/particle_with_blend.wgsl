struct VSOutput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) quad_position: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VSInput {
    @location(0) vert: vec2<f32>,
    @location(1) position: vec3<f32>,
}

@group(0) @binding(0) var prev_frame: texture_2d<f32>;
@group(0) @binding(1) var prev_sampler: sampler;
@group(0) @binding(2) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(in: VSInput) -> VSOutput {
    var out: VSOutput;

    let scale = 1.0;
    let final_pos = in.position.xy + in.vert * scale;

    out.frag_coord = vec4(final_pos, 0.0, 1.0);
    out.quad_position = in.vert;
    out.color = in.position * 0.5 + 0.5;

    return out;
}

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32> {
    // Map frag_coord from clip space to UV space [0, 1]
    let uv = (in.frag_coord.xy * 0.5 + 0.5);

    // Sample the previous frame texture
    let prev_color = textureSample(prev_frame, prev_sampler, uv);

    // Fade the previous frame (reduce alpha)
    let faded_color = vec4(prev_color.rgb, prev_color.a * 0.90);

    // Compute the distance from the center of the particle
    let dist = length(in.quad_position);

    // Alpha multiplier based on particle distance
    let alpha_mult = smoothstep(0.003, 0.001, dist);

    // New particle color with alpha multiplier
    let new_color = vec4(in.color.xy, 0.5, alpha_mult);

    // Blend the new particle on top of the faded previous frame
    return mix(faded_color, new_color, new_color.a);
}
