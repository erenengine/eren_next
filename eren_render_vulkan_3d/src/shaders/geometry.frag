#version 450

layout(location = 0) in vec3 vNormal;
layout(location = 1) in vec3 vWorldPos;
layout(location = 2) in vec4 vShadowPos;

layout(location = 0) out vec4 outColor;

layout(set = 1, binding = 0) uniform sampler2DShadow uShadow;

layout(set = 0, binding = 0) uniform CameraUBO {
  mat4 viewProj;
  mat4 lightViewProj;
  vec3 lightDir;
} uCam;

void main() {
  vec3 N = normalize(vNormal);
  vec3 L = normalize(-uCam.lightDir);
  float diff = max(dot(N, L), 0.0);

  vec3 proj = vShadowPos.xyz / vShadowPos.w;
  proj = proj * 0.5 + 0.5;
  float visibility = texture(uShadow, proj);
  float lightTerm = diff * visibility;

  vec3 albedo = vec3(0.7);
  outColor = vec4(albedo * lightTerm, 1.0);
}
