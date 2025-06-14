struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-0.025, -0.0333), // bottom-left
        vec2<f32>( 0.025, -0.0333), // bottom-right
        vec2<f32>(-0.025,  0.0333), // top-left
        vec2<f32>( 0.025,  0.0333), // top-right
    );

    var out: VertexOutput;
    let pos = positions[idx];
    out.position = vec4<f32>(pos.x, pos.y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
