use glam::Vec3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::{Camera, CameraUBO};
use crate::depth::create_depth_view;
use crate::types::{Vertex, GResult};
use crate::pipeline_cache::PipelineCache;
use crate::context::GfxContext;
use crate::camera_bind::CameraBind;

pub struct Renderer {
    pub ctx: GfxContext,

    pipeline: Option<wgpu::RenderPipeline>,
    vbuf: wgpu::Buffer,
    vcount: u32,

    cam: CameraBind,

    pipeline_cache: PipelineCache,
    pipeline_layout: wgpu::PipelineLayout,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Self {
        let ctx = GfxContext::new(window);
        
        // Adapter + device
        let adapter = pollster::block_on(ctx.instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&ctx.surface),
            force_fallback_adapter: false,
        })).expect("No adapter found");

        // Swapchain config
        let caps = ctx.surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        ctx.surface.configure(&ctx.device, &config);

        let cam = CameraBind::new(&ctx.device);

        // Shader + pipeline
        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&cam.bgl],
            push_constant_ranges: &[],
        });

        let pipeline_cache = PipelineCache::new();

        // Vertex buffer
        let verts: [Vertex; 3] = [
            Vertex { pos: [-0.6, -0.5], col: [1.0, 0.2, 0.2] },
            Vertex { pos: [ 0.6, -0.5], col: [0.2, 1.0, 0.2] },
            Vertex { pos: [ 0.0,  0.6], col: [0.2, 0.2, 1.0] },
        ];

        let vbuf = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vbuf"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vcount = verts.len() as u32;

        Self {
            ctx,
            pipeline: None,
            vbuf, vcount,
            cam,
            pipeline_cache,
            pipeline_layout,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return; }
        self.ctx.config.width = new_size.width;
        self.ctx.config.height = new_size.height;
        self.ctx.surface.configure(&self.ctx.device, &self.ctx.config);
        self.ctx.depth_view = create_depth_view(&self.ctx.device, self.ctx.config.width, self.ctx.config.height);
    }

    pub fn render(&mut self) -> GResult<()> {
        let frame = self.ctx.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("triangle"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.06, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.ctx.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Some(ref p) = self.pipeline {
                rp.set_pipeline(p);
                rp.set_bind_group(0, &self.cam.bind_group, &[]);
                rp.set_vertex_buffer(0, self.vbuf.slice(..));
                rp.draw(0..self.vcount, 0..1);
            }
        }

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn render_with<F>(&mut self, mut extra_pass: F) -> GResult<()>
    where
        F: FnMut(&mut wgpu::CommandEncoder, &wgpu::TextureView),
    {
        let frame = self.ctx.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("triangle"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.06, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.ctx.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Some(ref p) = self.pipeline {
                rp.set_pipeline(p);
                rp.set_bind_group(0, &self.cam.bind_group, &[]);
                rp.set_vertex_buffer(0, self.vbuf.slice(..));
                rp.draw(0..self.vcount, 0..1);
            }
            rp.set_bind_group(0, &self.cam.bind_group, &[]);
            rp.set_vertex_buffer(0, self.vbuf.slice(..));
            rp.draw(0..self.vcount, 0..1);
        }

        extra_pass(&mut encoder, &view);

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn aspect(&self) -> f32 {
        self.ctx.config.width as f32 / self.ctx.config.height as f32
    }

    pub fn update_camera_ubo(&mut self, ubo: &crate::CameraUBO) {
        self.ctx.queue.write_buffer(&self.cam.buffer, 0, bytemuck::bytes_of(ubo));
    }

    pub fn rebuild_pipeline(
        &mut self,
        shader_src: &shader_core::WgslSource,
        overrides: shader_core::Overrides,
        topo: shader_core::Topology,
    ) {
        let state = shader_core::RenderState {
            format: self.ctx.config.format,
            depth: true,
            msaa: 1,
            topo,
        };
        let key = shader_core::ShaderKey::new(shader_src, state, &overrides);
        let p = self.pipeline_cache.get_or_create(
            key,
            &self.ctx.device,
            &self.pipeline_layout,
            shader_src,
            &state,
            &overrides,
            &[crate::types::Vertex::layout()],
        ).clone();
        self.pipeline = Some(p);
    }

    pub fn build_pipeline(
        &mut self,
        shader_src: &shader_core::WgslSource,
        state: &shader_core::RenderState<wgpu::TextureFormat>,
        overrides: &shader_core::Overrides,
        vertex_layouts: &[wgpu::VertexBufferLayout<'static>],
    ) {
        let key = shader_core::ShaderKey::new(shader_src, *state, overrides);
        let p = self.pipeline_cache.get_or_create(
            key,
            &self.ctx.device,
            &self.pipeline_layout,
            shader_src,
            state,
            overrides,
            vertex_layouts,
        ).clone();
        self.pipeline = Some(p);
    }
}
