// Vertex buffer (triangle verticies)
struct Vertex {
    @location(0) vertex: vec2f
}

struct GlobalContext {
    texel: vec2f, // Size of a texel
    camera: vec2f, // Position of the camera
}

// Individual sprites
struct Sprite {
    pos: vec2f, // Position in the world
}

// Position and texture UV mappings and color adjustments
struct VertexOutput {
    @builtin(position) position:vec4f, // Vertex position translated to screen space (-1 .. 1)
    @location(0) uv: vec2f,  // UV coordinates on the texture (0..1)
}

// Shared between all objects
@group(0) @binding(0) var<uniform> world:GlobalContext;
@group(0) @binding(1) var sampl: sampler;

// Specific to a texture / batch
@group(1) @binding(0) var text: texture_2d<f32>;
@group(1) @binding(1) var<storage, read> sprites:array<Sprite>;

@vertex
fn vs_main (
    input:Vertex,
    @builtin(instance_index) instance: u32
) -> VertexOutput {
    let sprite = sprites[instance];

    let pos = sprite.pos + -world.camera + input.vertex;
    let world_space = pos * world.texel;

    var output: VertexOutput;
    output.position = vec4f(world_space, 0.0, 1.0);
    var uv = input.vertex + vec2f(0.5, 0.5); // Move from -0.5..0.5 to 0..1;
    uv.y = 1.0 - uv.y;
    output.uv = uv;

    return output;
}

@fragment 
fn fs_main(vertex:VertexOutput) -> @location(0) vec4f {
    return textureSample(text, sampl, vertex.uv);
}