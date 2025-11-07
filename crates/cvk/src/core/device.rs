use std::ffi::{CStr, CString};

use ash::vk;

use crate::core::instance::{Instance, Surface};

pub struct DeviceExtensions {
    pub swapchain: Option<ash::khr::swapchain::Device>,
}

pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,

    pub main_queue: Queue,
    pub present_queue: Queue,

    pub command_pool: vk::CommandPool,

    pub extensions: DeviceExtensions,
}

impl Device {
    fn check_physical_device(
        physical_device: vk::PhysicalDevice,
        instance: &Instance,
        required_extensions: &Vec<*const i8>,
    ) -> Option<(u32, u32)> {
        let surface = instance.surface.as_ref();
        let instance = &instance.instance;

        let queue_family_count =
            unsafe { instance.get_physical_device_queue_family_properties2_len(physical_device) };
        let mut queue_families = vec![vk::QueueFamilyProperties2::default(); queue_family_count];
        unsafe {
            //instance.get_physical_device_properties2(physical_device, &mut props);
            //instance.get_physical_device_features2(physical_device, &mut features);
            instance
                .get_physical_device_queue_family_properties2(physical_device, &mut queue_families);
        }

        let extension_names = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .ok()?
        }
        .iter()
        .map(|prop| CString::from(unsafe { CStr::from_ptr(prop.extension_name.as_ptr()) }))
        .collect::<Vec<_>>();

        for &ext in required_extensions.iter() {
            let ext_cstr = CString::from(unsafe { CStr::from_ptr(ext) });
            if !extension_names.contains(&ext_cstr) {
                return None;
            }
        }

        let graphics_families = queue_families
            .iter()
            .enumerate()
            .filter_map(|(i, queue_family)| {
                queue_family
                    .queue_family_properties
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS)
                    .then_some(i as u32)
            })
            .collect::<Vec<u32>>();

        if let Some(Surface {
            handle: surface,
            fns: surface_fns,
            ..
        }) = surface
        {
            let present_families = queue_families
                .iter()
                .enumerate()
                .filter_map(|(i, _)| {
                    if let Ok(true) = unsafe {
                        surface_fns.get_physical_device_surface_support(
                            physical_device,
                            i as u32,
                            *surface,
                        )
                    } {
                        Some(i as u32)
                    } else {
                        None
                    }
                })
                .collect::<Vec<u32>>();

            let combined_familes: Vec<u32> = graphics_families
                .iter()
                .filter_map(|&idx| present_families.contains(&idx).then_some(idx))
                .collect();

            if let Some(&idx) = combined_familes.first() {
                return Some((idx, idx));
            } else {
                return Some((*graphics_families.first()?, *present_families.first()?));
            }
        } else {
            let &idx = graphics_families.first()?;

            return Some((idx, idx));
        }
    }

    pub fn new(instance: &Instance) -> Self {
        let mut required_extensions = vec![];

        if instance.surface.is_some() {
            required_extensions.push(ash::khr::swapchain::NAME.as_ptr());
        }

        for physical_device in unsafe {
            instance
                .instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        } {
            if let Some((main_idx, present_idx)) =
                Self::check_physical_device(physical_device, instance, &required_extensions)
            {
                let queue_infos: Vec<_> = if main_idx == present_idx {
                    vec![main_idx]
                } else {
                    vec![main_idx, present_idx]
                }
                .iter()
                .map(|&idx| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(idx)
                        .queue_priorities(&[1.0])
                })
                .collect();

                let mut features2 = vk::PhysicalDeviceFeatures2::default();

                let device_info = vk::DeviceCreateInfo::default()
                    .queue_create_infos(queue_infos.as_slice())
                    .enabled_extension_names(&required_extensions)
                    .push_next(&mut features2);

                let device = unsafe {
                    instance
                        .instance
                        .create_device(physical_device, &device_info, None)
                }
                .expect("Failed to create device");

                let main_queue = Queue {
                    handle: unsafe {
                        device.get_device_queue2(
                            &vk::DeviceQueueInfo2::default()
                                .queue_family_index(main_idx)
                                .queue_index(0),
                        )
                    },
                    family_idx: main_idx,
                };

                let present_queue = Queue {
                    handle: unsafe {
                        device.get_device_queue2(
                            &vk::DeviceQueueInfo2::default()
                                .queue_family_index(present_idx)
                                .queue_index(0),
                        )
                    },
                    family_idx: present_idx,
                };

                let extensions = DeviceExtensions {
                    swapchain: instance
                        .surface
                        .is_some()
                        .then(|| ash::khr::swapchain::Device::new(&instance.instance, &device)),
                };

                let command_pool_info = vk::CommandPoolCreateInfo::default()
                    .queue_family_index(main_idx)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

                let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }
                    .expect("Failed to create command pool");

                return Self {
                    physical_device,
                    device,
                    main_queue,
                    present_queue,
                    command_pool,
                    extensions,
                };
            }
        }
        panic!("Failed to find a suitable physical device");
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        println!("dropping the device");
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
        }
    }
}

#[derive(cvk_macros::VkHandle)]
pub struct Queue {
    pub handle: vk::Queue,
    pub family_idx: u32,
}
