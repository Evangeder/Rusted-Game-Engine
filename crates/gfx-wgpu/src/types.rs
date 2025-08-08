use bytemuck::{Pod, Zeroable};

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub col: [f32; 3],
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { shader_location: 0, offset: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { shader_location: 1, offset: 8, format: wgpu::VertexFormat::Float32x3 },
            ],
        }
    }
}

pub type GResult<T> = Result<T, wgpu::SurfaceError>;
