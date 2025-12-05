use std::ffi::{CStr, CString};

use utils::{Build, Buildable};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::UIContext;

const APP_NAME: &'static CStr = c"Caustix Viewer";
const ENGINE_NAME: &'static CStr = c"Caustix";

struct AppData {
    _ui_context: UIContext,
    _pipeline: cvk::GraphicsPipeline,
    _render_pass: utils::Shared<cvk::RenderPass>
}

pub struct App {
    name: CString,
    engine_name: CString,

    app_data: Option<AppData>,
}

impl App {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let window_attribs = Window::default_attributes()
            .with_title(self.name.to_string_lossy())
            .with_inner_size(LogicalSize::new(640, 480))
            .with_resizable(false);

        let window = event_loop.create_window(window_attribs).unwrap();

        let swapchain_info = cvk::SwapchainInfo::default()
            .add_image_usage(cvk::ImageUsage::COLOR_ATTACHMENT)
            .push_preferred_format(cvk::Format::R8G8B8A8_SRGB);

        let context_info = cvk::ContextInfo::default()
            .app_name(self.name.clone())
            .engine_name(self.engine_name.clone())
            .version(cvk::ApiVersion::V1_2)
            .debugging(cfg!(debug_assertions))
            .window(window)
            .swapchain_info(swapchain_info);

        cvk::Context::init(context_info);

        let vertex_shader = cvk::Shader::builder()
            .stage(cvk::ShaderStage::VERTEX)
            .glsl_file("assets/shaders/tri_vert.glsl")
            .build();

        let fragment_shader = cvk::Shader::builder()
            .stage(cvk::ShaderStage::FRAGMENT)
            .glsl_file("assets/shaders/tri_frag.glsl")
            .build();

        let image = cvk::Image::builder()
            .extent_2d((1280, 720))
            .format(cvk::Format::R8G8B8A8_UNORM)
            .usage(cvk::ImageUsage::COLOR_ATTACHMENT)
            .memory_usage(cvk::MemoryUsage::PreferDevice)
            .build();

        dbg!(image.extent());

        let shared_image = image.share();

        dbg!(&shared_image);

        let subresource = cvk::ImageSubresource::default().aspect(cvk::ImageAspect::COLOR);
        let image_view = cvk::ImageView::new(&shared_image, &subresource);

        dbg!(&image_view);

        let render_pass = cvk::RenderPass::builder()
            .push_attachment(cvk::Attachment::swapchain())
            .build().share();

        let layout = cvk::PipelineLayout::build();

        let pipeline = cvk::GraphicsPipeline::builder()
            .push_shader(vertex_shader)
            .push_shader(fragment_shader)
            .render_pass(&render_pass)
            .layout(layout)
            .build();

        self.app_data = Some(AppData {
            _ui_context: UIContext::new(),
            _pipeline: pipeline,
            _render_pass: render_pass
        });
    }

    fn redraw(&mut self) {
    }

    fn handle_event(&mut self, _event: WindowEvent, _event_loop: &ActiveEventLoop) {
        /*
        let ui_context = match self.app_data.as_mut() {
            Some(app_data) => &mut app_data.ui_context,
            None => return,
        };

        let cvk_context = Context::get();

        let window = match cvk_context.window() {
            Some(window) => window,
            None => return
        };

        let _ = ui_context.winit_state.on_window_event(window, &event);

        */
    }

    pub fn run() {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        {
            let mut app = App {
                name: APP_NAME.into(),
                engine_name: ENGINE_NAME.into(),
                app_data: None,
            };

            event_loop.run_app(&mut app).unwrap();
        }

        cvk::Context::destroy();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            other => {
                if let Some(window) = cvk::Context::get().window() {
                    match other {
                        WindowEvent::RedrawRequested => {
                            self.redraw();
                            window.request_redraw();
                        }
                        event => self.handle_event(event, event_loop),
                    }
                }
            }
        }
    }
}
