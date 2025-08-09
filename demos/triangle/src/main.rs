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
    angle: f32,
    last_frame: Instant,
}

impl Demo {
    fn new() -> Self {
        let inner = platform::App::new(|_win| { });
        Self {
            inner,
            renderer: None,
            ui: None,
            rot_speed: 1.0,
            angle: 0.0,
            last_frame: Instant::now(),
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
            let renderer = gfx_wgpu::Renderer::new(win);
            let ui = gfx_wgpu::UiLayer::new(
                win,
                &renderer.device,
                &renderer.queue,
                renderer.config.format
            );
            self.ui = Some(ui);
            self.renderer = Some(renderer);
            self.renderer.as_mut().unwrap().update_camera(0.0);
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
                    let device = renderer.device.clone();
                    let queue  = renderer.queue.clone();

                    let _ = renderer.render_with(|enc, view| {
                        ui.build_and_render(
                            win, &device, &queue, enc, view,
                            |ui| {
                                ui.slider("Delta Time Scale", 0.0, 5.0, &mut local_speed);
                            }
                        );
                    });

                    self.rot_speed = local_speed;
                    self.angle += delta_time * self.rot_speed;
                    renderer.update_camera(self.angle);
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
