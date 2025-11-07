use std::u64;

use ash::vk;

use crate::Context;

#[derive(cvk_macros::VkHandle)]
pub struct Fence(vk::Fence);


impl Fence {
    pub fn new(signaled: bool) -> Self {

        let flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        };

        let info = vk::FenceCreateInfo::default()
            .flags(flags);

        let handle = unsafe { Context::get_device().create_fence(&info, None) }.expect("Failed to create fence");

        Self(handle)
    }

    pub fn wait_with_timeout(&self, timeout: u64) {
        unsafe { Context::get_device().wait_for_fences(&[self.0], true, timeout) }.expect("Failed to wait for fence");
    }

    pub fn wait(&self) {
        self.wait_with_timeout(u64::MAX);
    }

    pub fn reset(&self) {
        unsafe { Context::get_device().reset_fences(&[self.0])}.expect("Failed to reset fence");
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe { Context::get_device().destroy_fence(self.0, None) };
    }
}


#[derive(cvk_macros::VkHandle)]
pub struct Semaphore(vk::Semaphore);

impl Semaphore {
    pub fn new() -> Self {
        let info = vk::SemaphoreCreateInfo::default();

        let handle = unsafe { Context::get_device().create_semaphore(&info, None) }.expect("Failed to create semaphore");

        Self(handle)
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe { Context::get_device().destroy_semaphore(self.0, None) };
    }
}