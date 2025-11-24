pub mod command_buffer;
pub mod context;
mod device;
pub mod extent;
mod instance;
pub mod swapchain;

pub use command_buffer::*;
pub use context::*;
pub use extent::*;
pub use swapchain::*;

use crate::ImageAspect;

pub trait VkHandle {
    type HandleType;

    fn handle(&self) -> Self::HandleType;
}

pub type Format = ash::vk::Format;

pub fn infer_aspect_from_format(format: Format) -> ImageAspect {
    match format {
        Format::D16_UNORM | Format::D32_SFLOAT | Format::X8_D24_UNORM_PACK32 => ImageAspect::DEPTH,
        Format::S8_UINT => ImageAspect::STENCIL,
        Format::D24_UNORM_S8_UINT | Format::D32_SFLOAT_S8_UINT => {
            ImageAspect::DEPTH | ImageAspect::STENCIL
        }
        _ => ImageAspect::COLOR,
    }
}
