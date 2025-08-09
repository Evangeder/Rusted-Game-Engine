use std::collections::HashMap;
use shader_core::{WgslSource, RenderState, ShaderKey, Topology, Overrides};
use wgpu::{Device, PipelineLayout, TextureFormat};

pub struct PipelineCache {
    map: HashMap<ShaderKey<TextureFormat>, wgpu::RenderPipeline>,
}

impl PipelineCache {
    pub fn new() -> Self { Self { map: HashMap::new() } }

    pub fn get_or_create(
        &mut self,
        key: ShaderKey<TextureFormat>,
        device: &Device,
        layout: &PipelineLayout,
        src: &WgslSource,
        state: &RenderState<TextureFormat>,
        overrides: &Overrides,
        vertex_layouts: &[wgpu::VertexBufferLayout<'static>],
    ) -> &wgpu::RenderPipeline {
        self.map.entry(key.clone()).or_insert_with(|| {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(src.name),
                source: wgpu::ShaderSource::Wgsl(src.code.into()),
            });

            let topo = match state.topo {
                Topology::TriangleList  => wgpu::PrimitiveTopology::TriangleList,
                Topology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
                Topology::LineList      => wgpu::PrimitiveTopology::LineList,
            };

            // --- NEW: z HashMap -> posortowana vec i wycinek (&[(&str, f64)])
            // kopiujemy, sortujemy po kluczu żeby hash/klucz był deterministyczny
            let mut pairs_owned: Vec<(String, f64)> =
                overrides.map.iter().map(|(k,v)| (k.clone(), *v)).collect();
            pairs_owned.sort_by(|a,b| a.0.cmp(&b.0));
            // teraz robimy wektor referencji (&str, f64); ważne: odnosi się do pairs_owned
            let pairs_ref: Vec<(&str, f64)> =
                pairs_owned.iter().map(|(k,v)| (k.as_str(), *v)).collect();

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("pso:{}", src.name)),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    buffers: vertex_layouts,
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &pairs_ref,
                        zero_initialize_workgroup_memory: true,
                        ..Default::default()
                    },
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &pairs_ref,
                        zero_initialize_workgroup_memory: true,
                        ..Default::default()
                    },
                }),
                primitive: wgpu::PrimitiveState { topology: topo, ..Default::default() },
                depth_stencil: if state.depth {
                    Some(wgpu::DepthStencilState {
                        format: crate::types::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    })
                } else { None },
                multisample: wgpu::MultisampleState { count: state.msaa, ..Default::default() },
                multiview: None,
                cache: None,
            })
        })
    }

}
