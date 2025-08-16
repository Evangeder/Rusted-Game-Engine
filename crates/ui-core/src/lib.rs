pub trait Ui {
    fn text(&self, txt: &str);
    fn separator(&self);
    fn checkbox(&self, label: &str, value: &mut bool) -> bool;
    fn slider_f32(&self, label: &str, range: std::ops::RangeInclusive<f32>, value: &mut f32) -> bool;
    fn button(&self, label: &str) -> bool;

    fn window(
        &self,
        title: &str,
        size: [f32; 2],
        build: &mut dyn for<'a> FnMut(&'a dyn Ui),
    );
}

pub trait UiBackend {
    type Device;
    type Queue;
    type Encoder;
    type View;

    fn handle_window_event(
        &mut self,
        window: &winit::window::Window,
        id: winit::window::WindowId,
        event: &winit::event::WindowEvent,
    );

    fn build_and_render(
        &mut self,
        window: &winit::window::Window,
        device: &Self::Device,
        queue: &Self::Queue,
        encoder: &mut Self::Encoder,
        view: &Self::View,
        build: &mut dyn for<'a> FnMut(&'a dyn Ui),
    );
}
