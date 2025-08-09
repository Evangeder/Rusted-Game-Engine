#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
};

struct Demo {
    inner: platform::App,
    renderer: Option<gfx_wgpu::Renderer>,
    ui: Option<gfx_wgpu::UiLayer>,
    rot_speed: f32,
    camera: gfx_wgpu::Camera,
    angle: f32,
    last_frame: Instant,
    fog: bool,
    shader_src: Option<shader_core::WgslSource>,
}

impl Demo {
    fn new() -> Self {
        let inner = platform::App::new(|_win| { });
        Self {
            inner,
            renderer: None,
            ui: None,
            rot_speed: 1.0,
            camera: gfx_wgpu::Camera::new(glam::f32::Vec3::new(1.5, 1.5, 2.5), glam::f32::Vec3::ZERO),
            angle: 0.0,
            last_frame: Instant::now(),
            fog: true,
            shader_src: None,
        }
    }
}

impl ApplicationHandler for Demo {
    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(win) = &self.inner.window {
            win.request_redraw();
        }
    }

    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.inner.resumed(el);
        el.set_control_flow(ControlFlow::Poll);

        if let Some(win) = &self.inner.window {
            let mut renderer = gfx_wgpu::Renderer::new(win);

            let src = shader_core::WgslSource {
                name: "triangle.wgsl",
                code: include_str!("../shaders/triangle.wgsl"),
            };

            let state = shader_core::RenderState {
                format: renderer.ctx.config.format,
                depth: true,
                msaa: 1,
                topo: shader_core::Topology::TriangleList,
            };

            let mut ov = shader_core::Overrides::default();
            ov.set_bool("USE_FOG", true);
            ov.set_f32("TINT_R", 1.0);
            ov.set_f32("TINT_G", 0.9);
            ov.set_f32("TINT_B", 0.9);

            renderer.build_pipeline(&src, &state, &ov, &[gfx_wgpu::Vertex::layout()]);

            self.shader_src = Some(src);

            let ui = gfx_wgpu::UiLayer::new(
                win, &renderer.ctx.device, &renderer.ctx.queue, renderer.ctx.config.format
            );
            self.ui = Some(ui);
            self.renderer = Some(renderer);

            let aspect = self.renderer.as_ref().unwrap().aspect();
            let ubo = self.camera.make_mvp(aspect, 0.0);
            self.renderer.as_mut().unwrap().update_camera_ubo(&ubo);
            win.request_redraw();
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: winit::window::WindowId, event: WindowEvent) {
        self.inner.window_event(el, id, event.clone());

        if let (Some(win), Some(renderer), Some(ui)) = (&self.inner.window, self.renderer.as_mut(), self.ui.as_mut()) {
            ui.handle_window_event(win, id, &event);

            match event {
                WindowEvent::Resized(sz) => { renderer.resize(sz); win.request_redraw(); }
                WindowEvent::RedrawRequested => {
                    let now = Instant::now();
                    let delta_time = now.duration_since(self.last_frame).as_secs_f32();
                    self.last_frame = now;

                    let mut local_speed = self.rot_speed;
                    let device = renderer.ctx.device.clone();
                    let queue  = renderer.ctx.queue.clone();

                    let mut apply_overrides = false;

                    let _ = renderer.render_with(|enc, view| {
                        ui.build_and_render(
                            win, &device, &queue, enc, view,
                            |ui| {
                                ui.slider("Delta Time Scale", 0.0, 5.0, &mut local_speed);
                                ui.checkbox("Fog", &mut self.fog);
                                if ui.button("Apply shader overrides") {
                                    apply_overrides = true;
                                }
                            }
                        );
                    });

                    if apply_overrides {
                        let mut new_ov = shader_core::Overrides::default();
                        new_ov.set_bool("USE_FOG", self.fog);
                        new_ov.set_f32("TINT_R", 1.0);
                        new_ov.set_f32("TINT_G", 0.9);
                        new_ov.set_f32("TINT_B", 0.9);

                        let src = match &self.shader_src {
                            Some(s) => s,
                            None => panic!("shader_src not set")
                        };

                        renderer.rebuild_pipeline(src, new_ov, shader_core::Topology::TriangleList);
                    }

                    self.rot_speed = local_speed;
                    self.angle += delta_time * self.rot_speed;

                    let aspect = renderer.aspect();
                    let ubo = self.camera.make_mvp(aspect, self.angle);
                    renderer.update_camera_ubo(&ubo);
                }

                WindowEvent::Occluded(false) | WindowEvent::Focused(true) => win.request_redraw(),
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Demo::new();
    platform::run(&mut app)?;
    Ok(())
}
