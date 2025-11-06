
use ash::vk;
use vk_mem::Alloc;

use crate::VkHandle;

use utils::{Buildable, Build};

pub struct Buffer {
    handle: vk::Buffer,
    allocation: vk_mem::Allocation,

    size: vk::DeviceSize,
}

impl Buffer {

}

impl VkHandle for Buffer {

    type HandleType = vk::Buffer;

    fn handle(&self) -> Self::HandleType {
        self.handle
    }
}

impl Buildable for Buffer {
    type Builder = BufferBuilder;
}

pub struct BufferBuilder {

}

impl Default for BufferBuilder {
    fn default() -> Self {
        Self {  }
    }
}

impl Build for BufferBuilder {
    type Target = Buffer;

    fn build(&self) -> Self::Target {
        todo!()
    }
}