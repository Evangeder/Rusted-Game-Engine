use std::collections::HashMap;
use shader_core::*;
use wgpu::{Device, ShaderModule, PipelineLayout, TextureFormat};

pub struct WgpuShaderBackend<'a> {
    pub device: &'a Device,
    pub modules: HashMap<&'static str, ShaderModule>,
}

impl<'a> WgpuShaderBackend<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self { device, modules: HashMap::new() }
    }

    fn get_or_create_module(&mut self, src: &WgslSource) -> &ShaderModule {
        self.modules.entry(src.name).or_insert_with(|| {
            self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(src.name),
                source: wgpu::ShaderSource::Wgsl(src.code.into()),
            })
        })
    }

    fn tf_from_u32(fmt: u32) -> TextureFormat {
        unsafe { std::mem::transmute::<u32, TextureFormat>(fmt) }
    }

    fn constants_vec(overrides: &Overrides) -> Vec<wgpu::PipelineConstant> {
        let mut out = Vec::with_capacity(overrides.map.len());
        for (k, v) in overrides.map.iter() {
            let value = match v {
                Constant::Bool(b) => (*b as u32).into(),
                Constant::F32(f) => (*f).into(),
                Constant::F32x3(v) => wgpu::PipelineConstantValue::Float32x3(*v),
                Constant::U32(u) => (*u).into(),
                Constant::I32(i) => (*i as u32).into(),
            };
            out.push(wgpu::PipelineConstant { name: k, value });
        }
        out
    }

    fn topo(t: Topology) -> wgpu::PrimitiveTopology {
        match t {
            Topology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            Topology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
            Topology::LineList => wgpu::PrimitiveTopology::LineList,
        }
    }
}

impl<'a> ShaderBackend for WgpuShaderBackend<'a> {
    type Pipeline = wgpu::RenderPipeline;

    fn compile_pipeline(
        &mut self,
        src: &WgslSource,
        state: &RenderState,
        overrides: &Overrides,
        layouts: &[&BackendBindLayout],
    ) -> Self::Pipeline {
        let module = self.get_or_create_module(src);
        let constants = Self::constants_vec(overrides);

        unreachable!("In this version, pass PipelineLayout from outside")
    }
}
