use imgui::{Context, Ui};
use imgui_wgpu::{Renderer as ImGuiRenderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{event::WindowEvent, window::Window};
use winit::event::Event;

pub struct UiLayer {
    pub imgui: Context,
    platform: WinitPlatform,
    renderer: ImGuiRenderer,
}

impl UiLayer {
    pub fn new(window: &Window, device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::new(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);

        let font_size = (13.0 * window.scale_factor() as f32).max(12.0);
        imgui.io_mut().font_global_scale = 1.0 / window.scale_factor() as f32;
        imgui.fonts().add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig { size_pixels: font_size, ..Default::default() })
        }]);

        let config = RendererConfig { texture_format: surface_format, ..Default::default() };
        let renderer = ImGuiRenderer::new(&mut imgui, device, queue, config);

        Self { imgui, platform, renderer }
    }

    pub fn handle_window_event(&mut self, window: &Window, window_id: winit::window::WindowId, event: &WindowEvent) {
        let ev: Event<()> = Event::WindowEvent {
            window_id,
            event: event.clone(),
        };
        self.platform.handle_event(self.imgui.io_mut(), window, &ev);
    }

    pub fn build_and_render<F>(&mut self, window: &Window, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, mut ui_content: F) where F: FnMut(&Ui) {
        self.platform.prepare_frame(self.imgui.io_mut(), window).ok();
        let ui = self.imgui.frame();

        ui.window("Camera")
            .size([300.0, 220.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text("Camera controls");
                ui.separator();
                ui_content(&ui);
            });

        self.platform.prepare_render(&ui, window);
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

        self.renderer
            .render(&draw_data, queue, device, &mut rpass)
            .expect("imgui render");
    }
}
