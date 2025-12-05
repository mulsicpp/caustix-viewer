use ash::vk;
use utils::{Build, Buildable, Share};

use crate::{Context, Format, PipelineLayout, RenderPass, Shader};

#[derive(cvk_macros::VkHandle, utils::Share, Debug)]
pub struct GraphicsPipeline {
    handle: vk::Pipeline,
    layout: utils::Shared<PipelineLayout>,
    render_pass: utils::Shared<RenderPass>,
}

impl GraphicsPipeline {

    #[inline]
    pub const fn layout(&self) -> &utils::Shared<PipelineLayout> {
        &self.layout
    }

    #[inline]
    pub const fn render_pass(&self) -> &utils::Shared<RenderPass> {
        &self.render_pass
    }
}

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
    #[no_param]
    render_pass: Option<utils::Shared<RenderPass>>,
    #[no_param]
    layout: Option<utils::Shared<PipelineLayout>>,
}

impl GraphicsPipelineBuilder {
    #[inline]
    pub fn render_pass(mut self, render_pass: impl Share<Internal = RenderPass>) -> Self {
        self.render_pass = Some(render_pass.share());
        self
    }

    #[inline]
    pub fn layout(mut self, layout: impl Share<Internal = PipelineLayout>) -> Self {
        self.layout = Some(layout.share());
        self
    }
}

impl Default for GraphicsPipelineBuilder {
    fn default() -> Self {
        Self {
            shaders: vec![],
            render_pass: None,
            layout: None,
        }
    }
}

impl Build for GraphicsPipelineBuilder {
    type Target = GraphicsPipeline;

    fn build(&self) -> Self::Target {
        let shader_infos: Vec<_> = self.shaders.iter().map(|shader| shader.to_vk()).collect();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let swapchain_extent = Context::get().swapchain().unwrap().extent();
        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_extent.to_vk())];
        let viewports = [vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];
        let viewport_info = vk::PipelineViewportStateCreateInfo::default()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::default()
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .depth_clamp_enable(false)
            .depth_bias_enable(false)
            .rasterizer_discard_enable(false);

        let multisample_info = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)];

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&blend_attachments)
            .blend_constants([0.0; 4]);

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&[]);

        let render_pass = self
            .render_pass
            .as_ref()
            .expect("Graphics pipeline needs a render pass")
            .clone();

        let layout = self
            .layout
            .as_ref()
            .map(|layout| layout.clone())
            .unwrap_or(PipelineLayout::build().share());

        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_infos)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .render_pass(render_pass.handle())
            .subpass(0)
            .layout(layout.handle())
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(0);

        let handle = unsafe {
            Context::get_device().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[create_info],
                None,
            )
        }
        .expect("Failed to create graphics pipeline")[0];

        GraphicsPipeline {
            handle,
            layout,
            render_pass,
        }
    }
}

pub struct VertexInputInfo {}

pub type VertexInputRate = vk::VertexInputRate;

#[derive(Clone, Debug, utils::Paramters)]
pub struct VertexBindingInfo {
    binding: u32,
    stride: vk::DeviceSize,
    input_rate: VertexInputRate,
    #[vec(push_attribute)]
    attributes: Vec<VertexAttributeInfo>,
}

#[derive(Copy, Clone, Debug, utils::Paramters)]
pub struct VertexAttributeInfo {
    location: u32,
    format: Format,
    offset: vk::DeviceSize,
}

impl VertexAttributeInfo {
    pub fn new(location: u32, format: Format, offset: vk::DeviceSize) -> Self {
        Self {
            location,
            format,
            offset,
        }
    }
}

pub struct RasterizationInfo {}
