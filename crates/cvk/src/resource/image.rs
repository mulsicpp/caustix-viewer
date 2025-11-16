use ash::vk::{self, Format};
use utils::{Build, Buildable};
use vk_mem::Alloc;

use crate::{Context, Extent2D, MemoryUsage};

pub use vk::{ImageLayout, ImageTiling, ImageUsageFlags as ImageUsage};

#[derive(cvk_macros::VkHandle, utils::Share, Debug)]
pub struct Image {
    handle: vk::Image,
    allocation: vk_mem::Allocation,

    format: Format,
    extent: Extent2D,
}

impl Image {
    #[inline]
    pub const fn format(&self) -> Format {
        self.format
    }

    #[inline]
    pub const fn extent(&self) -> Extent2D {
        self.extent
    }
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
    type Builder<'a> = ImageBuilder;
}

#[derive(utils::Paramters, Clone, Debug)]
pub struct ImageBuilder {
    format: Format,
    extent: Extent2D,
    tiling: ImageTiling,

    #[flag]
    usage: ImageUsage,
    memory_usage: MemoryUsage,
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self {
            format: vk::Format::UNDEFINED,
            extent: Extent2D {
                width: 1,
                height: 1,
            },
            tiling: ImageTiling::OPTIMAL,

            usage: ImageUsage::empty(),
            memory_usage: MemoryUsage::Auto,
        }
    }
}

impl Build for ImageBuilder {
    type Target = Image;

    fn build(&self) -> Self::Target {
        assert!(!self.usage.is_empty(), "Image usage connot be empty");
        assert_ne!(
            self.format,
            vk::Format::UNDEFINED,
            "Image format connot be UNDEFINED"
        );

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(self.format)
            .extent(self.extent.to_vk_3d())
            .tiling(self.tiling)
            .usage(self.usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .mip_levels(1)
            .array_layers(1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: self.memory_usage.as_vma(),
            ..Default::default()
        };

        let (handle, allocation) = unsafe {
            Context::get()
                .allocator()
                .create_image(&image_info, &alloc_info)
        }
        .expect("Failed to create image");

        Image {
            handle,
            allocation,

            format: self.format,
            extent: self.extent,
        }
    }
}
