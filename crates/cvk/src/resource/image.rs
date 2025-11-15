use std::path::PathBuf;

use ash::vk::{self, Format};
use utils::{Build, Buildable};

use crate::{BufferRegion, Context, MemoryUsage};

pub use vk::{Extent2D, ImageLayout, ImageTiling, ImageUsageFlags as ImageUsage};

#[derive(cvk_macros::VkHandle, Debug)]
pub struct Image {
    handle: vk::Image,
    allocation: vk_mem::Allocation,
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            Context::get()
                .allocator()
                .destroy_image(self.handle, &mut self.allocation);
        }
    }
}

impl Buildable for Image {
    type Builder<'a> = ImageBuilder<'a>;
}



#[derive(Debug, Clone)]
pub enum ImageData<'a> {
    File(PathBuf),
    Buffer(BufferRegion<'a, u8>),
    Bytes(&'a [u8]),
    None
}

#[derive(utils::Paramters, Clone, Debug)]
pub struct ImageBuilder<'a> {
    format: Format,
    extent: Extent2D,
    tiling: ImageTiling,

    data: ImageData<'a>,

    usage: ImageUsage,
    memory_usage: MemoryUsage
}

impl Default for ImageBuilder<'_> {
    fn default() -> Self {
        Self {
            format: vk::Format::UNDEFINED,
            extent: Extent2D {
                width: 1,
                height: 1,
            },
            tiling: ImageTiling::OPTIMAL,

            data: ImageData::None,

            usage: ImageUsage::empty(),
            memory_usage: MemoryUsage::Auto
        }
    }
}

impl Build for ImageBuilder<'_> {
    type Target = Image;

    fn build(&self) -> Self::Target {
        todo!()
    }
}
