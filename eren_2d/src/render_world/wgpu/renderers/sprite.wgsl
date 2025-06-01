struct ScreenInfo {
    resolution: vec2<f32>,
    scale_factor: f32,
};

@group(0) @binding(0)
var<uniform> screen: ScreenInfo;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,

    @location(2) size: vec2<f32>,
    @location(3) mat0: vec3<f32>,
    @location(4) mat1: vec3<f32>,
    @location(5) mat2: vec3<f32>,
    @location(6) alpha: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_uv: vec2<f32>,
    @location(1) frag_alpha: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let resolution = screen.resolution;
    let scale_factor = screen.scale_factor;

    let local_pos = (input.pos - vec2<f32>(0.5, 0.5)) * input.size;

    let local = vec3<f32>(local_pos, 1.0);

    let mat = mat3x3<f32>(
        input.mat0,
        input.mat1,
        input.mat2,
    );
    let world = mat * local;

    let world_scaled = world.xy * scale_factor;
    let pos_ndc = (world_scaled / resolution) * 2.0;
    let ndc_final = vec2<f32>(pos_ndc.x, -pos_ndc.y);

    out.position = vec4<f32>(ndc_final, 0.0, 1.0);
    out.frag_uv = input.uv;
    out.frag_alpha = input.alpha;
    return out;
}

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(sprite_texture, sprite_sampler, in.frag_uv);
    return vec4<f32>(color.rgb, color.a * in.frag_alpha);
}
