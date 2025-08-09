use glam::Vec3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::{Camera, CameraUBO};
use crate::depth::create_depth_view;
use crate::types::{Vertex, DEPTH_FORMAT, GResult};
use shader_core::{WgslSource, RenderState, Topology, Overrides, ShaderKey};
use crate::pipeline_cache::PipelineCache;

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    pipeline: wgpu::RenderPipeline,
    vbuf: wgpu::Buffer,
    vcount: u32,

    depth_view: wgpu::TextureView,
    cam_buf: wgpu::Buffer,
    cam_bg: wgpu::BindGroup,

    camera: Camera,

    pipeline_cache: PipelineCache,
    pipeline_layout: wgpu::PipelineLayout,
    shader_src: WgslSource,
    current_overrides: Overrides,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Self {
        // Instance and surface
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = unsafe {
            instance
                .create_surface_unsafe(
                    wgpu::SurfaceTargetUnsafe::from_window(window).expect("get raw window handle")
                )
                .expect("create surface")
        };

        // Adapter + device
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).expect("No adapter found");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            }
        )).expect("request_device failed");

        // Swapchain config
        let caps = surface.get_capabilities(&adapter);
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
        surface.configure(&device, &config);

        // Depth
        let depth_view = create_depth_view(&device, config.width, config.height);

        // Camera: instance + BGL + BG + UBO
        let camera = Camera::new(Vec3::new(1.5, 1.5, 2.5), Vec3::ZERO);

        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&camera_bgl],
            push_constant_ranges: &[],
        });

        let cam_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera_ubo"),
            size: std::mem::size_of::<CameraUBO>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cam_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: cam_buf.as_entire_binding(),
            }],
        });

        // Shader + pipeline
        let shader_src = WgslSource {
            name: "triangle.wgsl",
            code: include_str!("../shaders/triangle.wgsl"),
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&camera_bgl],
            push_constant_ranges: &[],
        });

        let state = RenderState {
            format,
            depth: true,
            msaa: 1,
            topo: Topology::TriangleList,
        };

        let mut pipeline_cache = PipelineCache::new();
        let overrides = Overrides::default(); // startowo bez fog/tint
        let key = ShaderKey::new(&shader_src, state, &overrides);
        let pipeline = pipeline_cache.get_or_create(
            key,
            &device,
            &pipeline_layout,
            &shader_src,
            &state,
            &overrides,
            &[Vertex::layout()]
        ).clone();

        // Vertex buffer
        let verts: [Vertex; 3] = [
            Vertex { pos: [-0.6, -0.5], col: [1.0, 0.2, 0.2] },
            Vertex { pos: [ 0.6, -0.5], col: [0.2, 1.0, 0.2] },
            Vertex { pos: [ 0.0,  0.6], col: [0.2, 0.2, 1.0] },
        ];

        let vbuf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vbuf"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vcount = verts.len() as u32;

        Self {
            surface, device, queue, config,
            pipeline, vbuf, vcount,
            depth_view, cam_buf, cam_bg,
            camera,
            pipeline_cache,
            pipeline_layout,
            shader_src,
            current_overrides: overrides,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return; }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_view = create_depth_view(&self.device, self.config.width, self.config.height);
    }

    pub fn render(&mut self) -> GResult<()> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });

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
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.cam_bg, &[]);
            rp.set_vertex_buffer(0, self.vbuf.slice(..));
            rp.draw(0..self.vcount, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn render_with<F>(&mut self, mut extra_pass: F) -> GResult<()>
    where
        F: FnMut(&mut wgpu::CommandEncoder, &wgpu::TextureView),
    {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });

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
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.cam_bg, &[]);
            rp.set_vertex_buffer(0, self.vbuf.slice(..));
            rp.draw(0..self.vcount, 0..1);
        }

        extra_pass(&mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn update_camera(&mut self, t: f32) {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let ubo = self.camera.make_mvp(aspect, t);
        self.queue.write_buffer(&self.cam_buf, 0, bytemuck::bytes_of(&ubo));
    }

    pub fn rebuild_pipeline(&mut self, overrides: Overrides, topo: Topology) {
        self.current_overrides = overrides.clone();

        let state = RenderState {
            format: self.config.format,
            depth: true,
            msaa: 1,
            topo,
        };
        let key = ShaderKey::new(&self.shader_src, state, &self.current_overrides);

        let p = self.pipeline_cache.get_or_create(
            key,
            &self.device,
            &self.pipeline_layout,
            &self.shader_src,
            &state,
            &self.current_overrides,
            &[crate::types::Vertex::layout()],
        ).clone();

        self.pipeline = p;
    }
}
