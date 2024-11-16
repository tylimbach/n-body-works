struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) quad_position: vec2<f32>,
};

struct Uniforms {
    resolution: vec2<f32>
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    @location(0) vert: vec2<f32>,
    @location(1) position: vec3<f32>,
) -> VSOutput {
    var out: VSOutput;

    let scale = 1.0;
    let final_pos = position.xy + vert * scale;

    out.position = vec4(final_pos, 0.0, 1.0);
    out.quad_position = vert;

    return out;
}

@fragment
fn fs_main(input: VSOutput) -> @location(0) vec4<f32> {
    let dist = length(input.quad_position);

    if dist > 0.01 {
      discard;
    }

    let alpha = smoothstep(1.0, 0.9, dist);

    return vec4(1.0, 1.0, 0.0, alpha);
}
