use egui::Context as EguiContext;
use egui_ash_renderer::{Options as EguiOptions, Renderer as EguiRenderer};
use egui_winit::State as EguiWinitState;
use utils::{Build, Buildable};

pub struct UIContext {
    pub winit_state: EguiWinitState,
    pub renderer: EguiRenderer,
    pub render_pass: cvk::RenderPass,
}

impl UIContext {
    pub fn new() -> Self {
        let context = EguiContext::default();

        let cvk_context = cvk::Context::get();
        let window = cvk_context.window().unwrap();

        let viewport_id = context.viewport_id();

        let winit_state = EguiWinitState::new(
            context,
            viewport_id,
            window,
            None,
            Some(winit::window::Theme::Dark),
            None,
        );

        let render_pass = cvk::RenderPass::builder()
            .push_attachment(cvk::Attachment::swapchain())
            .build();

        let options = EguiOptions {
            in_flight_frames: 1,
            enable_depth_test: false,
            enable_depth_write: false,
            srgb_framebuffer: true,
        };

        let renderer = EguiRenderer::with_default_allocator(
            &cvk_context.instance().instance,
            cvk_context.device().physical_device,
            cvk_context.device().device.clone(),
            render_pass.handle(),
            options,
        ).unwrap();

        Self {
            winit_state,
            renderer,
            render_pass,
        }
    }
}