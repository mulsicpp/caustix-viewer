pub mod command_buffer;
pub mod context;
mod device;
mod instance;

pub use command_buffer::*;
pub use context::*;



pub trait VkHandle {
    type HandleType;

    fn handle(&self) -> Self::HandleType;
}


use ash::vk;

pub use vk::Format;

#[derive(Clone, Copy, Debug, utils::Paramters)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

impl Extent2D {
    #[inline]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub const fn to_vk(&self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.width,
            height: self.height,
        }
    }

    #[inline]
    pub const fn to_vk_3d(&self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: 1,
        }
    }
}

impl From<(u32, u32)> for Extent2D {
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}

impl From<[u32; 2]> for Extent2D {
    fn from([width, height]: [u32; 2]) -> Self {
        Self { width, height }
    }
}
