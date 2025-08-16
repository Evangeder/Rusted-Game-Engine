use imgui::{Context, Ui as ImUi};
use imgui_wgpu::{Renderer as ImGuiRenderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{event::WindowEvent, window::Window};
use winit::event::Event;

struct UiAdapter<'a> {
    ui: &'a ImUi,
}

impl<'a> ui_core::Ui for UiAdapter<'a> {
    fn text(&self, txt: &str) { self.ui.text(txt); }
    fn separator(&self) { self.ui.separator(); }
    fn checkbox(&self, label: &str, v: &mut bool) -> bool { self.ui.checkbox(label, v) }
    fn slider_f32(&self, label: &str, range: std::ops::RangeInclusive<f32>, v: &mut f32) -> bool {
        let (min, max) = range.into_inner();
        self.ui.slider(label, min, max, v)
    }
    fn button(&self, label: &str) -> bool { self.ui.button(label) }

    fn window(&self, title: &str, size: [f32; 2], build: &mut dyn FnMut(&dyn ui_core::Ui)) {
        self.ui
            .window(title)
            .size(size, imgui::Condition::FirstUseEver)
            .build(|| {
                let adapter = UiAdapter { ui: self.ui };
                build(&adapter as &dyn ui_core::Ui);
            });
    }
}

pub struct UiLayer {
    pub imgui: Context,
    platform: WinitPlatform,
    renderer: ImGuiRenderer,
}

impl UiLayer {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::new(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);

        let font_size = (13.0 * window.scale_factor() as f32).max(12.0);
        imgui.io_mut().font_global_scale = 1.0 / window.scale_factor() as f32;
        imgui.fonts().add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig { size_pixels: font_size, ..Default::default() }),
        }]);

        let config = RendererConfig { texture_format: surface_format, ..Default::default() };
        let renderer = ImGuiRenderer::new(&mut imgui, device, queue, config);

        Self { imgui, platform, renderer }
    }
}

impl ui_core::UiBackend for UiLayer {
    type Device = wgpu::Device;
    type Queue = wgpu::Queue;
    type Encoder = wgpu::CommandEncoder;
    type View = wgpu::TextureView;

    fn handle_window_event(
        &mut self,
        window: &Window,
        window_id: winit::window::WindowId,
        event: &WindowEvent,
    ) {
        let ev: Event<()> = Event::WindowEvent { window_id, event: event.clone() };
        self.platform.handle_event(self.imgui.io_mut(), window, &ev);
    }

    fn build_and_render(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        build: &mut dyn for<'a> FnMut(&'a dyn ui_core::Ui), // <- HRTB
    ) {
        self.platform.prepare_frame(self.imgui.io_mut(), window).ok();

        let frame = self.imgui.frame();
        let ui_ref: &ImUi = &*frame;
        let adapter = UiAdapter { ui: ui_ref };

        build(&adapter as &dyn ui_core::Ui);

        self.platform.prepare_render(ui_ref, window);
        let draw_data = self.imgui.render();

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("imgui"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer.render(&draw_data, queue, device, &mut rpass).expect("imgui render");
    }
}
