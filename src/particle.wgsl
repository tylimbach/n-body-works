struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) quad_position: vec2<f32>,
    @location(1) debug_color: vec3<f32>,
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
    
    // Make particles much bigger for debugging
    let size = 0.5;

    let final_pos = position.xy + (vert * size);

    out.position = vec4(final_pos, 0.0, 1.0);
    out.quad_position = vert;
    
    // Debug color based on position
    out.debug_color = vec3(
        (position.x + 1.0) * 0.5,  // R: x mapped to 0-1
        (position.y + 1.0) * 0.5,  // G: y mapped to 0-1
        0.5                             // B: constant
    );

    return out;
}

@fragment
fn fs_main(input: VSOutput) -> @location(0) vec4<f32> {
    // Disable circle masking for debug
    return vec4(input.debug_color, 1.0);
}
