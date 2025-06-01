#version 450

layout(set = 1, binding = 0) uniform sampler2D sprite_texture;

layout(location = 0) in vec2 frag_uv;
layout(location = 1) in float frag_alpha;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(sprite_texture, frag_uv);
    out_color = vec4(color.rgb, color.a * frag_alpha);
}
