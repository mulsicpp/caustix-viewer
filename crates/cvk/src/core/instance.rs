use std::ffi::{CStr, CString, c_void};

use ash::vk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

use crate::ContextInfo;

pub struct Instance {
    pub debug_utils: Option<DebugUtils>,
    pub surface: Option<Surface>,
    pub instance: ash::Instance,
    _entry: ash::Entry,
}

impl Instance {
    const VALIDATION_LAYER: &'static CStr = &c"VK_LAYER_KHRONOS_validation";

    unsafe extern "system" fn debug_callback(
        _severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        _type_flags: vk::DebugUtilsMessageTypeFlagsEXT,
        callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
        _user_data: *mut c_void,
    ) -> u32 {
        if let Some(msg) = unsafe { (*callback_data).message_as_c_str() } {
            println!("Validation Layer:\n {}", msg.to_string_lossy());
        }

        vk::FALSE
    }

    pub fn new(info: ContextInfo) -> Self {
        let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan entry") };

        let layer_names = unsafe { entry.enumerate_instance_layer_properties().unwrap() }
            .iter()
            .filter_map(|prop| Some(CString::from(prop.layer_name_as_c_str().ok()?)))
            .collect::<Vec<_>>();

        let extension_names =
            unsafe { entry.enumerate_instance_extension_properties(None).unwrap() }
                .iter()
                .filter_map(|prop| Some(CString::from(prop.extension_name_as_c_str().ok()?)))
                .collect::<Vec<_>>();

        let mut required_layers: Vec<*const i8> = vec![];
        let mut required_extensions: Vec<*const i8> = vec![];

        if let Some(ref window) = info.window {
            let raw_display_handle = window.display_handle().unwrap().as_raw();

            let mut surface_extenstions =
                ash_window::enumerate_required_extensions(raw_display_handle)
                    .expect("Failed to enumerate surface extensions")
                    .into_iter()
                    .map(|&raw| raw)
                    .collect::<Vec<_>>();

            required_extensions.append(&mut surface_extenstions);
        }

        if info.debugging {
            required_layers.push(Self::VALIDATION_LAYER.as_ptr());
            required_extensions.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        for &ext in required_extensions.iter() {
            let ext_cstr = CString::from(unsafe { CStr::from_ptr(ext) });
            if !extension_names.contains(&ext_cstr) {
                panic!(
                    "The required extension '{}' is not supported",
                    ext_cstr.to_string_lossy()
                );
            }
        }

        for &layer in required_layers.iter() {
            let layer_cstr = CString::from(unsafe { CStr::from_ptr(layer) });
            if !layer_names.contains(&layer_cstr) {
                panic!(
                    "The required layer '{}' is not present",
                    layer_cstr.to_string_lossy()
                );
            }
        }

        let app_info = vk::ApplicationInfo::default()
            .application_name(&info.app_name)
            .application_version(0)
            .engine_name(&info.engine_name)
            .engine_version(0)
            .api_version(info.version as u32);

        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(required_layers.as_slice())
            .enabled_extension_names(required_extensions.as_slice());

        let mut debug_messenger_info = None;

        if info.debugging {
            use vk::DebugUtilsMessageSeverityFlagsEXT as Severity;
            use vk::DebugUtilsMessageTypeFlagsEXT as Type;

            debug_messenger_info = Some(vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(Severity::VERBOSE | Severity::WARNING | Severity::ERROR)
                .message_type(Type::GENERAL | Type::PERFORMANCE | Type::VALIDATION)
                .pfn_user_callback(Some(Self::debug_callback)));
            instance_info = instance_info.push_next(debug_messenger_info.as_mut().unwrap());

        };

        let instance = unsafe {
            entry
                .create_instance(&instance_info, None)
                .expect("Failed to create VkInstance")
        };

        let debug_utils = if let Some(messenger_info) = debug_messenger_info {
            Some(DebugUtils::new(&entry, &instance, &messenger_info))
        } else {
            None
        };

        let surface = if let Some(window) = info.window {
            Some(Surface::new(&entry, &instance, window))
        } else {
            None
        };

        Self {
            debug_utils,
            surface,
            instance,
            _entry: entry,
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        println!("dropping the instance");
        unsafe {
            if let Some(DebugUtils { ref fns, messenger }) = self.debug_utils {
                fns.destroy_debug_utils_messenger(messenger, None);
            }

            if let Some(Surface {
                ref fns, handle, ..
            }) = self.surface
            {
                fns.destroy_surface(handle, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

pub struct DebugUtils {
    messenger: vk::DebugUtilsMessengerEXT,
    fns: ash::ext::debug_utils::Instance,
}

impl DebugUtils {
    fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        messenger_info: &vk::DebugUtilsMessengerCreateInfoEXT,
    ) -> Self {
        let fns = ash::ext::debug_utils::Instance::new(&entry, &instance);

        let messenger = unsafe { fns.create_debug_utils_messenger(messenger_info, None) }
            .expect("Failed to create debug messenger");

        Self { fns, messenger }
    }
}

#[derive(cvk_macros::VkHandle)]
pub struct Surface {
    pub(crate) handle: vk::SurfaceKHR,
    pub(crate) window: Window,
    pub(crate) fns: ash::khr::surface::Instance,
}

impl Surface {
    fn new(entry: &ash::Entry, instance: &ash::Instance, window: Window) -> Self {
        let display_handle = window
            .display_handle()
            .expect("Failed to acquire display handle")
            .as_raw();
        let window_handle = window
            .window_handle()
            .expect("Failed to acquire window handle")
            .as_raw();

        Self {
            handle: unsafe {
                ash_window::create_surface(entry, instance, display_handle, window_handle, None)
                    .expect("Failed to create surface")
            },
            window,
            fns: ash::khr::surface::Instance::new(&entry, &instance),
        }
    }
}