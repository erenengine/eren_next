#version 450

layout(location = 0) in vec3 inPos;

layout(set = 0, binding = 0) uniform LightVP {
  mat4 lightViewProj;
} uLight;

layout(push_constant) uniform Push {
  mat4 modelMatrix;
} uPush;

void main() {
  gl_Position = uLight.lightViewProj * uPush.modelMatrix * vec4(inPos, 1.0);
}
