use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId, WindowAttributes},
};

pub struct App {
    pub window: Option<Window>,
    pub on_resumed: Option<Box<dyn FnOnce(&Window)>>,
}

impl App {
    pub fn new(on_resumed: impl FnOnce(&Window) + 'static) -> Self {
        Self { window: None, on_resumed: Some(Box::new(on_resumed)) }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title("Hello, Engine")
            .with_inner_size(PhysicalSize::new(1280, 720)); // <= KONKRETNY TYP

        let window = event_loop.create_window(attrs).expect("create window");
        if let Some(cb) = self.on_resumed.take() {
            cb(&window);
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(win) = &self.window {
            if win.id() != id { return; }
        }
        match event {
            WindowEvent::CloseRequested => el.exit(),
            _ => {}
        }
    }
}

pub fn run<T: ApplicationHandler + 'static>(app: &mut T) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.run_app(app)
}
