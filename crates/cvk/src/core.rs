mod instance;
mod device;


use instance::*;
use device::*;

use std::sync::RwLock;
use std::sync::{OnceLock, RwLockReadGuard, RwLockWriteGuard};

use ash::vk;

use winit::window::Window;

use std::ffi::CString;

type ContextReadGuard = RwLockReadGuard<'static, Context>;
type ContextWriteGuard = RwLockWriteGuard<'static, Context>;

pub struct Context {
    _device: Device,
    surface: ash::vk::SurfaceKHR,
    window: Option<Window>,
    _instance: Instance,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum ApiVersion {
    V1_0 = vk::API_VERSION_1_0,
    V1_1 = vk::API_VERSION_1_1,
    V1_2 = vk::API_VERSION_1_2,
    V1_3 = vk::API_VERSION_1_3,
}

#[derive(utils::Paramters)]
pub struct ContextInfo {
    pub(crate) app_name: CString,
    pub(crate) engine_name: CString,
    pub(crate) version: ApiVersion,
    pub(crate) debugging: bool,
    pub(crate) window: Option<Window>,
}

impl Default for ContextInfo {
    fn default() -> Self {
        Self {
            app_name: CString::from(c"Vulkan App"),
            engine_name: CString::from(c"Engine"),
            version: ApiVersion::V1_3,
            debugging: false,
            window: None,
        }
    }
}

static CONTEXT: OnceLock<RwLock<Context>> = OnceLock::new();

impl Context {
    pub fn init(info: ContextInfo) {
        let instance = Instance::create(&info);

        let device = Device::create(&instance, &info);

        if let Err(_) = CONTEXT.set(RwLock::new(Context {
            _instance: instance,
            surface: vk::SurfaceKHR::null(),
            window: info.window,
            _device: device
        })) {
            panic!("Failed to initialize Vulkan context");
        }
    }

    pub fn get() -> ContextReadGuard {
        CONTEXT
            .get()
            .expect("Vulkan context is not initialized")
            .read()
            .unwrap()
    }

    pub fn try_get() -> Option<ContextReadGuard> {
        CONTEXT.get()?.read().ok()
    }

    pub fn get_mut() -> ContextWriteGuard {
        CONTEXT
            .get()
            .expect("Vulkan context is not initialized")
            .write()
            .unwrap()
    }

    pub fn try_get_mut() -> Option<ContextWriteGuard> {
        CONTEXT.get()?.write().ok()
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn window(&self) -> Option<&Window> {
        self.window.as_ref()
    }

    pub fn window_mut(&mut self) -> Option<&mut Window> {
        self.window.as_mut()
    }
}
