use std::ffi::{CStr, CString, c_void};

use ash::vk::{self, DebugUtilsMessengerEXT, Handle};
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};

use crate::ContextInfo;

pub(crate) struct InstanceExtensions {
    pub surface: Option<ash::khr::surface::Instance>,
    pub debug_utils: Option<ash::ext::debug_utils::Instance>,
}

pub struct Instance {
    debug_messenger: vk::DebugUtilsMessengerEXT,
    pub(crate) extensions: InstanceExtensions,
    pub(crate) instance: ash::Instance,
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

    pub fn create(info: &ContextInfo) -> Self {
        let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan entry") };

        let layer_names = unsafe { entry.enumerate_instance_layer_properties().unwrap() }
            .iter()
            .filter_map(|prop| Some(CString::from(prop.layer_name_as_c_str().ok()?)))
            .collect::<Vec<_>>();

        let extension_names = unsafe { entry.enumerate_instance_extension_properties(None).unwrap() }
            .iter()
            .filter_map(|prop| Some(CString::from(prop.extension_name_as_c_str().ok()?)))
            .collect::<Vec<_>>();

        let mut required_layers: Vec<*const i8> = vec![];
        let mut required_extensions: Vec<*const i8> = vec![];

        if let Some(ref window) = info.window {
            let raw_display_handle: RawDisplayHandle = window.display_handle().unwrap().into();

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

        let instance;
        let mut debug_utils = None;
        let mut debug_messenger = DebugUtilsMessengerEXT::null();

        if info.debugging {
            use vk::DebugUtilsMessageSeverityFlagsEXT as Severity;
            use vk::DebugUtilsMessageTypeFlagsEXT as Type;

            let mut debug_messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(Severity::VERBOSE | Severity::WARNING | Severity::ERROR)
                .message_type(Type::GENERAL | Type::PERFORMANCE | Type::VALIDATION)
                .pfn_user_callback(Some(Self::debug_callback));

            instance_info = instance_info.push_next(&mut debug_messenger_info);

            instance = unsafe {
                entry
                    .create_instance(&instance_info, None)
                    .expect("Failed to create VkInstance")
            };

            debug_utils = {
                let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);

                debug_messenger = unsafe {
                    debug_utils.create_debug_utils_messenger(&debug_messenger_info, None)
                }
                .expect("Failed to create debug messenger");

                Some(debug_utils)
            }
        } else {
            instance = unsafe {
                entry
                    .create_instance(&instance_info, None)
                    .expect("Failed to create VkInstance")
            };
        };

        let surface = info
            .window
            .as_ref()
            .and_then(|_| Some(ash::khr::surface::Instance::new(&entry, &instance)));

        let extensions = InstanceExtensions {
            surface,
            debug_utils,
        };

        Self {
            debug_messenger,
            extensions,
            instance,
            _entry: entry,
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        println!("dropping the instance");
        unsafe {
            if !self.debug_messenger.is_null() {
                if let Some(ref debug_utils) = self.extensions.debug_utils {
                    debug_utils.destroy_debug_utils_messenger(self.debug_messenger, None);
                } else {
                    ash::ext::debug_utils::Instance::new(&self._entry, &self.instance)
                        .destroy_debug_utils_messenger(self.debug_messenger, None);
                }
            }
            self.instance.destroy_instance(None);
        }
    }
}
