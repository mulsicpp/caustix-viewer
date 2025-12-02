use ash::vk;
use utils::{Build, Buildable};
use vk_mem::Alloc;

use crate::{Context, Extent2D, Extent3D, Format, MemoryUsage, Swapchain};

pub type ImageLayout = vk::ImageLayout;
pub type ImageTiling = vk::ImageTiling;
pub type ImageUsage = vk::ImageUsageFlags;
pub type ImageType = vk::ImageType;
pub type ImageFlags = vk::ImageCreateFlags;

#[derive(cvk_macros::VkHandle, utils::Share, Debug)]
pub struct Image {
    pub(crate) handle: vk::Image,
    pub(super) allocation: Option<vk_mem::Allocation>,

    format: Format,
    extent: Extent3D,
    image_type: ImageType,

    array_layers: u32,
    mip_levels: u32,

    flags: ImageFlags,
}

impl Image {
    pub(crate) fn swapchain_image(swapchain: &Swapchain, handle: vk::Image) -> Self {
        Self {
            handle,
            allocation: None,
            format: swapchain.format().format,
            extent: swapchain.extent().into(),
            image_type: ImageType::TYPE_2D,
            array_layers: 1,
            mip_levels: 1,
            flags: ImageFlags::empty(),
        }
    }

    #[inline]
    pub const fn format(&self) -> Format {
        self.format
    }

    #[inline]
    pub const fn extent(&self) -> Extent3D {
        self.extent
    }

    #[inline]
    pub const fn image_type(&self) -> ImageType {
        self.image_type
    }

    #[inline]
    pub const fn array_layers(&self) -> u32 {
        self.array_layers
    }

    #[inline]
    pub const fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    #[inline]
    pub const fn flags(&self) -> ImageFlags {
        self.flags
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            if let Some(mut allocation) = self.allocation {
                Context::get()
                    .allocator()
                    .destroy_image(self.handle, &mut allocation);
            }
        }
    }
}

impl Buildable for Image {
    type Builder<'a> = ImageBuilder;
}

#[derive(utils::Paramters, Clone, Debug)]
pub struct ImageBuilder {
    format: Format,
    extent: Extent3D,
    image_type: ImageType,
    tiling: ImageTiling,

    array_layers: u32,
    mip_levels: u32,

    #[flag(add_flag)]
    flags: ImageFlags,

    #[flag]
    usage: ImageUsage,
    memory_usage: MemoryUsage,
}

impl ImageBuilder {
    pub fn extent_1d(mut self, width: u32) -> Self {
        self.extent = width.into();
        self.image_type = ImageType::TYPE_1D;
        self
    }

    pub fn extent_2d(mut self, extent_2d: impl Into<Extent2D>) -> Self {
        self.extent = extent_2d.into().into();
        self.image_type = ImageType::TYPE_2D;
        self
    }

    pub fn extent_3d(mut self, extent_3d: impl Into<Extent3D>) -> Self {
        self.extent = extent_3d.into();
        self.image_type = ImageType::TYPE_3D;
        self
    }

    pub fn cube_map(self, width: u32) -> Self {
        self.add_flag(ImageFlags::CUBE_COMPATIBLE)
            .extent_2d((width, width))
            .array_layers(6u32)
    }
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self {
            format: vk::Format::UNDEFINED,
            image_type: ImageType::TYPE_2D,
            extent: (1, 1, 1).into(),
            tiling: ImageTiling::OPTIMAL,

            array_layers: 1,
            mip_levels: 1,

            flags: ImageFlags::empty(),

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
            .image_type(self.image_type)
            .format(self.format)
            .extent(self.extent.to_vk())
            .tiling(self.tiling)
            .usage(self.usage)
            .array_layers(self.array_layers)
            .mip_levels(self.mip_levels)
            .flags(self.flags)
            .samples(vk::SampleCountFlags::TYPE_1)
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
            allocation: Some(allocation),

            format: self.format,
            image_type: self.image_type,
            extent: self.extent,

            array_layers: self.array_layers,
            mip_levels: self.mip_levels,

            flags: self.flags,
        }
    }
}
