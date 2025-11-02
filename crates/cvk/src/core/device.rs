use ash::vk::{self, Handle};

use crate::core::instance::Instance;

pub struct Device {
    pub _physical_device: vk::PhysicalDevice,
    pub _device: ash::Device,

    pub _main_queue: vk::Queue,
    pub _present_queue: vk::Queue,
}

impl Device {
    fn select_physical_device(instance: &Instance) -> (vk::PhysicalDevice, u32) {
        let physical_devices =
            unsafe { instance.instance.enumerate_physical_devices() }.unwrap_or(vec![]);

        for physical_device in physical_devices {
            //let mut props = vk::PhysicalDeviceProperties2::default();
            //let mut features = vk::PhysicalDeviceFeatures2::default();

            let instance = &instance.instance;

            let queue_family_count = unsafe {
                instance.get_physical_device_queue_family_properties2_len(physical_device)
            };
            let mut queue_familys = vec![vk::QueueFamilyProperties2::default(); queue_family_count];

            unsafe {
                //instance.get_physical_device_properties2(physical_device, &mut props);
                //instance.get_physical_device_features2(physical_device, &mut features);
                instance.get_physical_device_queue_family_properties2(
                    physical_device,
                    &mut queue_familys,
                );
            }

            let mut main_queue_idx: Option<u32> = None;

            for (i, queue_family) in queue_familys.iter().enumerate() {
                if queue_family
                    .queue_family_properties
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS)
                {
                    main_queue_idx = Some(i as u32);
                }
            }

            if let Some(main_queue_idx) = main_queue_idx {
                return (physical_device, main_queue_idx);
            }
        }

        (vk::PhysicalDevice::null(), u32::MAX)
    }

    pub fn create(instance: &Instance) -> Self {
        let (physical_device, main_queue_idx) = Self::select_physical_device(instance);

        if physical_device.is_null() {
            panic!("Could not find suitable physical device");
        }

        let main_queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(main_queue_idx)
            .queue_priorities(&[1.0]);

        let queue_infos = vec![main_queue_info];

        let mut features2 = vk::PhysicalDeviceFeatures2::default();

        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(queue_infos.as_slice())
            .push_next(&mut features2);

        let device = unsafe {
            instance
                .instance
                .create_device(physical_device, &device_info, None)
        }
        .expect("Failed to create device");

        Self {
            _physical_device: physical_device,
            _device: device,
            _main_queue: vk::Queue::null(),
            _present_queue: vk::Queue::null(),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        println!("dropping the device");
        unsafe {
            self._device.destroy_device(None);
        }
    }
}
