use ash::vk;
use utils::{Build, Buildable};

use crate::Context;

#[derive(cvk_macros::VkHandle, utils::Share, Debug)]
pub struct PipelineLayout {
    handle: vk::PipelineLayout,
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            Context::get_device().destroy_pipeline_layout(self.handle, None);
        }
    }
}

impl Buildable for PipelineLayout {
    type Builder<'a> = PipelineLayoutBuilder;
}

#[derive(utils::Paramters, Clone, Debug)]
pub struct PipelineLayoutBuilder {}

impl Default for PipelineLayoutBuilder {
    fn default() -> Self {
        Self {  }
    }
}

impl Build for PipelineLayoutBuilder {
    type Target = PipelineLayout;

    fn build(&self) -> Self::Target {
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&[])
            .push_constant_ranges(&[]);

        let handle = unsafe { Context::get_device().create_pipeline_layout(&create_info, None) }
            .expect("Failed to create pipeline layout");

        PipelineLayout { handle }
    }
}
