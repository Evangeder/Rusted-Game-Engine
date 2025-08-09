override USE_FOG: bool = false;
override TINT_R: f32 = 1.0;
override TINT_G: f32 = 1.0;
override TINT_B: f32 = 1.0;

struct CameraUBO {
  mvp: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> cam: CameraUBO;

struct VsIn {
  @location(0) pos: vec2<f32>,
  @location(1) col: vec3<f32>,
};

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) col: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
  var o: VsOut;
  let tint = vec3<f32>(TINT_R, TINT_G, TINT_B);
  o.pos = cam.mvp * vec4<f32>(in.pos, 0.0, 1.0);
  o.col = in.col * tint;
  return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let fogged = mix(in.col, vec3(0.5, 0.6, 0.7), 0.3);
  let c = select(in.col, fogged, USE_FOG);
  return vec4<f32>(c, 1.0);
}
