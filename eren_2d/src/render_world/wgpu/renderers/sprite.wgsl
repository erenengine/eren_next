struct ScreenInfo {
    resolution: vec2<f32>,
    scale_factor: f32,
};

@group(0) @binding(0)
var<uniform> screen: ScreenInfo;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) offset: vec2<f32>,
    @location(3) size: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let resolution = screen.resolution;
    let scale_factor = screen.scale_factor;

    let physical_sprite_size = input.size * scale_factor;
    let quad_pos_pixels = input.pos * physical_sprite_size;

    let pos_ndc = quad_pos_pixels / resolution * 2.0;
    let offset_ndc = vec2<f32>(
        (input.offset.x / resolution.x * scale_factor) * 2.0,
        -(input.offset.y / resolution.y * scale_factor) * 2.0
    );

    out.position = vec4<f32>(offset_ndc + pos_ndc, 0.0, 1.0);
    out.frag_uv = input.uv;
    return out;
}

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(sprite_texture, sprite_sampler, in.frag_uv);
}
