#version 450

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUV;

layout(location = 0) out vec3 vNormal;
layout(location = 1) out vec3 vWorldPos;
layout(location = 2) out vec4 vShadowPos;

layout(set = 0, binding = 0) uniform CameraUBO {
  mat4 viewProj;
  mat4 lightViewProj;
  vec3 lightDir;
} uCam;

layout(push_constant) uniform Push {
  mat4 modelMatrix;
} uPush;

void main() {
  mat4 modelMatrix = uPush.modelMatrix;

  vec4 worldPos = modelMatrix * vec4(inPos, 1.0);
  vWorldPos     = worldPos.xyz;
  vNormal       = mat3(modelMatrix) * inNormal;
  vShadowPos    = uCam.lightViewProj * worldPos;

  gl_Position = uCam.viewProj * worldPos;
}
