use std::ffi::{CStr, CString};

use cvk::BufferUsage;
use utils::{Build, Buildable};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const APP_NAME: &'static CStr = c"Caustix Viewer";
const ENGINE_NAME: &'static CStr = c"Caustix";

pub struct App {
    name: CString,
    engine_name: CString,
}

impl App {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let window_attribs = Window::default_attributes()
            .with_title(self.name.to_string_lossy())
            .with_inner_size(LogicalSize::new(640, 480))
            .with_resizable(false);

        let window = event_loop.create_window(window_attribs).unwrap();

        let context_info = cvk::ContextInfo::default()
            .app_name(self.name.clone())
            .engine_name(self.engine_name.clone())
            .version(cvk::ApiVersion::V1_2)
            .debugging(cfg!(debug_assertions))
            .window(window);

        cvk::Context::init(context_info);

        let b1 = cvk::Buffer::builder()
            .data(&[5, 2])
            .usage(BufferUsage::TRANSFER_SRC | BufferUsage::TRANSFER_DST)
            .memory_usage(cvk::MemoryUsage::PreferDevice)
            .build();

        let b2_reg;

        {
            let b2 = cvk::Buffer::<i32>::builder()
                .staging_buffer()
                .usage(cvk::BufferUsage::TRANSFER_SRC | cvk::BufferUsage::TRANSFER_DST)
                .data(&[0, 1, 2, 3, 4])
                .build();

            let b2_slice = b2.mapped().unwrap();
            println!("{:?}", b2_slice);

            cvk::CommandBuffer::run_single_use(|recording| {
                recording.copy_buffer(&b1, &b2);
            });
            println!("{:?}", b2_slice);

            let reg = b2.region(1..);

            b2_reg = reg.region(..3);
            println!("{:?}",b2_reg.mapped().unwrap());
        }

    }

    fn redraw(&mut self) {}

    fn handle_event(&mut self, event: WindowEvent, _event_loop: &ActiveEventLoop) {
        // println!("event: {:#?}", event);
        match event {
            _ => (),
        }
    }

    pub fn run() {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App {
            name: APP_NAME.into(),
            engine_name: ENGINE_NAME.into(),
        };

        event_loop.run_app(&mut app).unwrap();

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
