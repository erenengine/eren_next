struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) mat0: vec4<f32>,
    @location(4) mat1: vec4<f32>,
    @location(5) mat2: vec4<f32>,
    @location(6) mat3: vec4<f32>,

    @location(7) alpha: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_uv: vec2<f32>,
    @location(1) frag_alpha: f32,
    @location(2) world_pos: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_matrix = mat4x4<f32>(
        input.mat0, input.mat1, input.mat2, input.mat3
    );
    let world_pos = world_matrix * vec4<f32>(input.position, 1.0);
    let world_normal = (world_matrix * vec4<f32>(input.normal, 0.0)).xyz;

    let pos_clip = camera.view_proj * world_pos;

    out.position = pos_clip;
    out.frag_uv = input.uv;
    out.frag_alpha = input.alpha;
    out.world_pos = world_pos.xyz;
    out.world_normal = normalize(world_normal);

    return out;
}

@group(1) @binding(0) var base_color_texture: texture_2d<f32>;
@group(1) @binding(1) var base_color_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(base_color_texture, base_color_sampler, input.frag_uv);
    return vec4<f32>(color.rgb, color.a * input.frag_alpha);
}
