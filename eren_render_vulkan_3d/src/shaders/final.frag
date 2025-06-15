#version 450

layout(location = 0) in vec2 vUV;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform sampler2D inputColor;

void main() {
  vec3 color = texture(inputColor, vUV).rgb;
  color = pow(color, vec3(1.0 / 2.2));
  outColor = vec4(color, 1.0);
}
