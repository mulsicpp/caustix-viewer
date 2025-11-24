use crate::Swapchain;
use crate::SwapchainInfo;

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

type DeviceReadGuard = MappedRwLockReadGuard<'static, ash::Device>;

pub struct Context {
    glsl_compiler: shaderc::Compiler,
    swapchain: Option<Swapchain>,
    allocator: vk_mem::Allocator,
    device: Device,
    instance: Instance,
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
    pub swapchain_info: Option<SwapchainInfo>,
}

impl Default for ContextInfo {
    fn default() -> Self {
        Self {
            app_name: CString::from(c"Vulkan App"),
            engine_name: CString::from(c"Engine"),
            version: ApiVersion::V1_3,
            debugging: false,
            window: None,
            swapchain_info: None,
        }
    }
}

static CONTEXT: RwLock<Option<Context>> = RwLock::new(None);

impl Context {
    pub fn init(info: ContextInfo) {
        let swapchain_info = info.swapchain_info.clone();
        let instance = Instance::new(info);

        let device = Device::new(&instance);

        let allocator_info = vk_mem::AllocatorCreateInfo::new(
            &instance.instance,
            &device.device,
            device.physical_device,
        );

        let swapchain = if let Some(info) = swapchain_info.as_ref() {
            let surface = instance
                .surface
                .as_ref()
                .expect("Swapchain was requested, but no surface is present");

            Some(Swapchain::new(
                &device,
                surface,
                info,
            ))
        } else {
            None
        };

        let allocator = unsafe { vk_mem::Allocator::new(allocator_info) }
            .expect("Failed to create the allocator");

        let glsl_compiler = shaderc::Compiler::new().expect("Failed to create GLSL compiler");

        *CONTEXT.write() = Some(Context {
            swapchain,
            glsl_compiler,
            allocator,
            device,
            instance,
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

    pub fn get_device() -> DeviceReadGuard {
        MappedRwLockReadGuard::map(Self::get(), |context| &context.device.device)
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn allocator(&self) -> &vk_mem::Allocator {
        &self.allocator
    }

    pub fn glsl_compiler(&self) -> &shaderc::Compiler {
        &self.glsl_compiler
    }

    pub fn swapchain(&self) -> Option<&Swapchain> {
        self.swapchain.as_ref()
    }

    pub fn window(&self) -> Option<&Window> {
        Some(&self.instance.surface.as_ref()?.window)
    }

    pub fn window_mut(&mut self) -> Option<&mut Window> {
        Some(&mut self.instance.surface.as_mut()?.window)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.swapchain.as_ref().and_then(|swapchain| unsafe {
            self.device
                .extensions
                .swapchain
                .as_ref()?
                .destroy_swapchain(swapchain.handle(), None);
            Some(())
        });
    }
}
