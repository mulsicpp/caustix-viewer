use ash::vk::{self, ImageType};
use utils::{AnyRange, Span, ToSpan};

use crate::{Context, Image};

pub type ImageAspect = vk::ImageAspectFlags;
pub type ImageViewType = vk::ImageViewType;

#[derive(utils::Paramters, Clone, Debug)]
pub struct ImageSubresource {
    pub aspect: ImageAspect,
    pub array_layers: AnyRange<u32>,
    pub mip_levels: AnyRange<u32>,
}

impl ImageSubresource {
    pub fn new(
        aspect: ImageAspect,
        array_layers: impl Into<AnyRange<u32>>,
        mip_levels: impl Into<AnyRange<u32>>,
    ) -> Self {
        Self {
            aspect,
            array_layers: array_layers.into(),
            mip_levels: mip_levels.into(),
        }
    }

    pub fn to_vk(&self, image: &Image) -> vk::ImageSubresourceRange {
        let array_layer_span = self
            .array_layers
            .clone()
            .to_span(Span::new(0, image.array_layers()));
        let mip_level_span = self
            .mip_levels
            .clone()
            .to_span(Span::new(0, image.mip_levels()));

        vk::ImageSubresourceRange {
            aspect_mask: self.aspect,
            base_array_layer: array_layer_span.offset,
            layer_count: array_layer_span.count,
            base_mip_level: mip_level_span.offset,
            level_count: mip_level_span.count,
        }
    }
}

impl Default for ImageSubresource {
    fn default() -> Self {
        Self {
            aspect: ImageAspect::COLOR,
            array_layers: (..).into(),
            mip_levels: (..).into(),
        }
    }
}

#[derive(cvk_macros::VkHandle, utils::Share, Debug)]
pub struct ImageView {
    handle: vk::ImageView,
    image: utils::Shared<Image>,
}

impl ImageView {
    pub fn new(image: impl utils::Share<Internal = Image>, subresource: &ImageSubresource) -> Self {
        let image = image.share();
        Self::new_with_type(&image, Self::infer_type(image.image_type()), subresource)
    }

    fn infer_type(image_type: ImageType) -> ImageViewType {
        match image_type {
            ImageType::TYPE_1D => ImageViewType::TYPE_1D,
            ImageType::TYPE_2D => ImageViewType::TYPE_2D,
            ImageType::TYPE_3D => ImageViewType::TYPE_3D,
            _ => ImageViewType::TYPE_2D,
        }
    }

    pub fn new_with_type(
        image: impl utils::Share<Internal = Image>,
        view_type: ImageViewType,
        subresource: &ImageSubresource,
    ) -> Self {
        Self::new_with_type_from_device(&*Context::get_device(), image, view_type, subresource)
    }

    pub fn new_with_type_from_device(
        device: &ash::Device,
        image: impl utils::Share<Internal = Image>,
        view_type: ImageViewType,
        subresource: &ImageSubresource,
    ) -> Self {
        let image = image.share();

        let info = vk::ImageViewCreateInfo::default()
            .view_type(view_type)
            .image(image.handle())
            .format(image.format())
            .subresource_range(subresource.to_vk(&*image));

        let handle =
            unsafe { device.create_image_view(&info, None) }.expect("Failed to create image view");

        Self { handle, image }
    }

    #[inline]
    pub const fn image(&self) -> &utils::Shared<Image> {
        &self.image
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        if self.image.allocation.is_some() {
            unsafe {
                Context::get_device().destroy_image_view(self.handle, None);
            }
        }
    }
}
