use ash::vk;
use utils::{Build, Buildable};

use crate::{Context, Shader};

#[derive(cvk_macros::VkHandle, utils::Share, Debug)]
pub struct GraphicsPipeline {
    handle: vk::Pipeline,
}

impl GraphicsPipeline {}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            Context::get_device().destroy_pipeline(self.handle, None);
        }
    }
}

impl Buildable for GraphicsPipeline {
    type Builder<'a> = GraphicsPipelineBuilder;
}

#[derive(utils::Paramters, Clone, Debug)]
pub struct GraphicsPipelineBuilder {
    #[vec(push_shader)]
    shaders: Vec<utils::Shared<Shader>>,
}

impl Default for GraphicsPipelineBuilder {
    fn default() -> Self {
        Self { shaders: vec![] }
    }
}

impl Build for GraphicsPipelineBuilder {
    type Target = GraphicsPipeline;

    fn build(&self) -> Self::Target {
        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(0);

        let _handle = unsafe {
            Context::get_device().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[create_info],
                None,
            )
        }
        .expect("Failed to create graphics pipeline")[0];

        GraphicsPipeline {
            handle: vk::Pipeline::null(),
        }
    }
}
