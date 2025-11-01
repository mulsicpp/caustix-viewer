use std::ffi::{CStr, CString, c_void};

use ash::vk;
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};

use crate::ContextInfo;

pub(crate) struct DebugObjects {
    pub(crate) debug_utils: ash::ext::debug_utils::Instance,
    pub(crate) debug_messenger: vk::DebugUtilsMessengerEXT,
}

pub(crate) struct Instance {
    _entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) debug_objs: Option<DebugObjects>,
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

    pub(crate) fn create(info: &ContextInfo) -> Self {
        let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan entry") };

        let mut required_instance_extensions: Vec<&CStr> = vec![];

        if let Some(ref window) = info.window {
            let raw_display_handle: RawDisplayHandle = window.display_handle().unwrap().into();

            let mut surface_extenstions =
                ash_window::enumerate_required_extensions(raw_display_handle)
                    .expect("Failed to enumerate surface extensions")
                    .into_iter()
                    .map(|&raw| unsafe { CStr::from_ptr(raw) })
                    .collect();
            required_instance_extensions.append(&mut surface_extenstions);
        }

        if info.debugging {
            let layers = unsafe { entry.enumerate_instance_layer_properties().unwrap() }
                .into_iter()
                .filter_map(|layer_prop| {
                    Some(CString::from(layer_prop.layer_name_as_c_str().ok()?))
                })
                .collect::<Vec<_>>();

            dbg!(layers.contains(&Self::VALIDATION_LAYER.into()));

            required_instance_extensions.push(ash::ext::debug_utils::NAME);
        }

        let instance_extensions =
            unsafe { entry.enumerate_instance_extension_properties(None).unwrap() }
                .into_iter()
                .filter_map(|ext_prop| {
                    Some(CString::from(ext_prop.extension_name_as_c_str().ok()?))
                })
                .collect::<Vec<_>>();

        dbg!(&required_instance_extensions);

        for ext in required_instance_extensions.iter() {
            if !instance_extensions.contains(&CString::from(*ext)) {
                panic!(
                    "The required extension '{}' is not supported",
                    ext.to_string_lossy()
                );
            }
        }

        let app_info = vk::ApplicationInfo::default()
            .application_name(&info.app_name)
            .application_version(0)
            .engine_name(&info.engine_name)
            .engine_version(0)
            .api_version(info.version as u32);

        let enabled_layers = [Self::VALIDATION_LAYER.as_ptr()];
        let enabled_extensions: Vec<_> = required_instance_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect();

        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(enabled_layers.as_slice())
            .enabled_extension_names(enabled_extensions.as_slice());

        let mut debug_messenger_info;

        if info.debugging {
            use vk::DebugUtilsMessageSeverityFlagsEXT as Severity;
            use vk::DebugUtilsMessageTypeFlagsEXT as Type;

            debug_messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(Severity::VERBOSE | Severity::WARNING | Severity::ERROR)
                .message_type(Type::GENERAL | Type::PERFORMANCE | Type::VALIDATION)
                .pfn_user_callback(Some(Self::debug_callback));

            instance_info = instance_info.push_next(&mut debug_messenger_info);
        }

        let instance = unsafe {
            entry
                .create_instance(&instance_info, None)
                .expect("Failed to create VkInstance")
        };

        let debug_objs = if info.debugging {
            Some(Self::create_debug_utils(&entry, &instance))
        } else {
            None
        };

        Self {
            _entry: entry,
            instance,
            debug_objs,
        }
    }

    fn create_debug_utils(entry: &ash::Entry, instance: &ash::Instance) -> DebugObjects {
        use vk::DebugUtilsMessageSeverityFlagsEXT as Severity;
        use vk::DebugUtilsMessageTypeFlagsEXT as Type;

        let debug_utils = ash::ext::debug_utils::Instance::new(entry, &instance);

        let debug_messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(Severity::VERBOSE | Severity::WARNING | Severity::ERROR)
            .message_type(Type::GENERAL | Type::PERFORMANCE | Type::VALIDATION)
            .pfn_user_callback(Some(Self::debug_callback));

        let debug_messenger =
            unsafe { debug_utils.create_debug_utils_messenger(&debug_messenger_info, None) }
                .expect("Failed to create debug messenger");

        DebugObjects {
            debug_utils,
            debug_messenger,
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        println!("dropping the instance");
        unsafe {
            if let Some(DebugObjects {
                ref debug_utils,
                debug_messenger,
            }) = self.debug_objs
            {
                debug_utils.destroy_debug_utils_messenger(debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
