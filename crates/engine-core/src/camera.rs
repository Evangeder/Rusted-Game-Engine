use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraUBO { pub mvp: [[f32; 4]; 4] }

#[derive(Clone)]
pub struct Camera {
    pub position: Vec3, pub target: Vec3, pub up: Vec3,
    pub fov_y: f32, pub z_near: f32, pub z_far: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov_y: 60f32.to_radians(),
            z_near: 0.1,
            z_far: 100.0,
        }
    }
    pub fn make_mvp(&self, aspect: f32, t: f32) -> CameraUBO { 
        let proj = Mat4::perspective_rh(self.fov_y, aspect, self.z_near, self.z_far);
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let model = Mat4::from_rotation_y(t);
        CameraUBO { mvp: (proj * view * model).to_cols_array_2d() }
    }
}