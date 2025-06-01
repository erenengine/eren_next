#version 450

layout(set = 0, binding = 0) uniform ScreenInfo {
    vec2 resolution;
    float scale_factor;
};

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_uv;

layout(location = 2) in vec2 in_size;
layout(location = 3) in vec3 in_mat0;
layout(location = 4) in vec3 in_mat1;
layout(location = 5) in vec3 in_mat2;
layout(location = 6) in float in_alpha;

layout(location = 0) out vec2 frag_uv;
layout(location = 1) out float frag_alpha;

void main() {
    vec2 local_pos = (in_pos - vec2(0.5)) * in_size;
    vec3 local = vec3(local_pos, 1.0);

    mat3 mat = mat3(in_mat0, in_mat1, in_mat2);
    vec3 world = mat * local;
    vec2 world_scaled = world.xy * scale_factor;
    vec2 pos_ndc = (world_scaled / resolution) * 2.0;
    vec2 ndc_final = vec2(pos_ndc.x, -pos_ndc.y);

    gl_Position = vec4(ndc_final, 0.0, 1.0);
    frag_uv = vec2(in_uv.x, 1.0 - in_uv.y);
    frag_alpha = in_alpha;
}
