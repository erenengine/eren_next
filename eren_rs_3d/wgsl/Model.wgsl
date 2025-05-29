struct Cam { vp : mat4x4<f32> };

@group(0) @binding(0) var<uniform> cam  : Cam;
@group(0) @binding(1) var samp : sampler;

struct Model { m : mat4x4<f32> };

@group(1) @binding(0) var<uniform> mdl  : Model;
@group(2) @binding(0) var tex : texture_2d<f32>;

struct VSIn  { @location(0) pos : vec3f, @location(1) nor : vec3f, @location(2) uv : vec2f };
struct VSOut { @builtin(position) pos : vec4f, @location(0) uv : vec2f };

@vertex
fn vs(input: VSIn) -> VSOut {
  var o: VSOut;
  o.pos = cam.vp * mdl.m * vec4f(input.pos, 1.0);
  o.uv  = input.uv;
  return o;
}

@fragment
fn fs(i: VSOut) -> @location(0) vec4f {
  return textureSample(tex, samp, i.uv);
}
