use super::device::*;
use super::instance::*;

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use ash::vk;

use winit::window::Window;

use std::ffi::CString;

type ContextReadGuard = MappedRwLockReadGuard<'static, Context>;
type ContextWriteGuard = MappedRwLockWriteGuard<'static, Context>;

pub struct Context {
    window: Option<Window>,
    _device: Device,
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
    pub app_name: CString,
    pub engine_name: CString,
    pub version: ApiVersion,
    pub debugging: bool,
    pub window: Option<Window>,
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

static CONTEXT: RwLock<Option<Context>> = RwLock::new(None);

impl Context {
    pub fn init(info: ContextInfo) {
        let instance = Instance::create(&info);

        // unsafe { instance.surface().unwrap().destroy_surface(vk::SurfaceKHR::null(), None) };

        let device = Device::create(&instance);

        *CONTEXT.write() = Some(Context {
            _instance: instance,
            _device: device,
            window: info.window,
        });
    }

    pub fn destroy() {
        *CONTEXT.write() = None;
    }

    pub fn get() -> ContextReadGuard {
        RwLockReadGuard::map(CONTEXT.read(), |context| {
            context.as_ref().expect("Vulkan context is not initialized")
        })
    }

    pub fn try_get() -> Option<ContextReadGuard> {
        RwLockReadGuard::try_map(CONTEXT.read(), |context| context.as_ref()).ok()
    }

    pub fn get_mut() -> ContextWriteGuard {
        RwLockWriteGuard::map(CONTEXT.write(), |context| {
            context.as_mut().expect("Vulkan context is not initialized")
        })
    }

    pub fn try_get_mut() -> Option<ContextWriteGuard> {
        RwLockWriteGuard::try_map(CONTEXT.write(), |context| context.as_mut()).ok()
    }

    pub fn window(&self) -> Option<&Window> {
        self.window.as_ref()
    }

    pub fn window_mut(&mut self) -> Option<&mut Window> {
        self.window.as_mut()
    }
}
