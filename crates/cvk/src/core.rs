pub mod command_buffer;
pub mod context;
pub mod extent;
mod device;
mod instance;

pub use command_buffer::*;
pub use context::*;
pub use extent::*;



pub trait VkHandle {
    type HandleType;

    fn handle(&self) -> Self::HandleType;
}


pub type Format = ash::vk::Format;