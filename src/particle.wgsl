struct Vertex {
  @location(0) position: vec3<f32>,
};

struct Uniforms {
  resolution: vec3<f32>,
};

struct VSOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) quad_position: vec3<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(vert: Vertex, @builtin(vertex_index) vertex_index: u32) -> VSOutput {
    let offsets = array<vec3<f32>, 6>(
        vec3<f32>(-1.0, -1.0, 0.0),
        vec3<f32>(1.0, -1.0, 0.0),
        vec3<f32>(-1.0, 1.0, 0.0),
        vec3<f32>(-1.0, 1.0, 0.0),
        vec3<f32>(1.0, -1.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
    );

    var offset: vec3<f32>;
    switch (vertex_index) {
        case 0u: { offset = offsets[0]; }
        case 1u: { offset = offsets[1]; }
        case 2u: { offset = offsets[2]; }
        case 3u: { offset = offsets[3]; }
        case 4u: { offset = offsets[4]; }
        case 5u: { offset = offsets[5]; }
        default: { offset = vec3<f32>(0.0, 0.0, 0.0); } // Should never hit this
    }

    var out: VSOutput;
    let SIZE : f32 = 10.0;
    let screen_space_offset = (offset * SIZE);
    out.position = vec4<f32>(vert.position + screen_space_offset, 1.0);
    out.quad_position = offset;
    return out;
}

@fragment
fn fs_main(input: VSOutput) -> @location(0) vec4<f32> {
    let dist = length(input.quad_position);
    if dist > 1.0 {
        discard;  // Discard fragments outside the unit circle
    }
    return vec4<f32>(1.0, 1.0, 0.0, 1.0);  // Yellow circle
}
