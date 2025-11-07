use ash::vk;


#[derive(cvk_macros::VkHandle)]
pub struct CommandBuffer {
    handle: vk::CommandBuffer,
}

impl CommandBuffer {

}