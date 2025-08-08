use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop},
};

struct Demo {
    inner: platform::App,
    renderer: Option<gfx_wgpu::Renderer>,
    start: Instant,
}

impl Demo {
    fn new() -> Self {
        let inner = platform::App::new(|_win| { });
        Self { inner, renderer: None, start: Instant::now() }
    }
}

impl ApplicationHandler for Demo {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.inner.resumed(el);
        if let Some(win) = &self.inner.window {
            self.renderer = Some(gfx_wgpu::Renderer::new(win));
            win.request_redraw();
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(win) = &self.inner.window {
            win.request_redraw();
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: winit::window::WindowId, event: WindowEvent) {
        self.inner.window_event(el, id, event.clone());

        if let (Some(win), Some(renderer)) = (&self.inner.window, self.renderer.as_mut()) {
            match event {
                WindowEvent::Resized(sz) => {
                    renderer.resize(sz);
                    win.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    let t = self.start.elapsed().as_secs_f32();
                    renderer.update_camera(t);
                    let _ = renderer.render();
                }
                WindowEvent::Occluded(false) | WindowEvent::Focused(true) => {
                    win.request_redraw();
                }
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
