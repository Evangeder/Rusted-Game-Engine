struct Camera {
  mvp: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> u_camera: Camera;

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@location(0) a_pos: vec2<f32>, @location(1) a_col: vec3<f32>) -> VSOut {
  var out: VSOut;
  let p = vec4<f32>(a_pos, 0.0, 1.0);
  out.pos = u_camera.mvp * p;
  out.color = a_col;
  return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
  return vec4<f32>(in.color, 1.0);
}
