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
    var out: VsOut;
    let p = vec4<f32>(in.pos, 0.0, 1.0);
    out.pos = cam.mvp * p;    // <— używamy kamery
    out.col = in.col;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.col, 1.0);
}
