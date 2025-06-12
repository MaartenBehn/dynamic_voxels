use std::marker::PhantomData;

use octa_force::{glam::UVec3, vulkan::{ash::vk::{PushConstantRange, ShaderStageFlags}, CommandBuffer, ComputePipeline, ComputePipelineCreateInfo, Context, DescriptorSetLayout, PipelineLayout}, OctaResult};



#[derive(Debug)]
pub struct ShaderStage {
    pub push_constant_range: PushConstantRange,
    pub pipeline_layout: PipelineLayout,
    pub pipeline: ComputePipeline,
}

impl ShaderStage {
    pub fn new(
        context: &Context, 
        shader_bin: &[u8], 
        descriptor_set_layouts: &[&DescriptorSetLayout],
        push_constant_size: u32,
    ) -> OctaResult<Self> {
        let push_constant_range = PushConstantRange::default()
            .offset(0)
            .size(push_constant_size)
            .stage_flags(ShaderStageFlags::COMPUTE);

        let pipeline_layout = context.create_pipeline_layout(
            descriptor_set_layouts,
            &[push_constant_range])?;

        let pipeline = context.create_compute_pipeline(
            &pipeline_layout,
            ComputePipelineCreateInfo {
                shader_source: shader_bin,
            },
        )?;
        
        Ok(Self{
            push_constant_range,
            pipeline_layout,
            pipeline,
        })
    }

    pub fn render<D>(&self, buffer: &CommandBuffer, dispatch_params: D, dispatch_size: UVec3) {
        buffer.bind_compute_pipeline(&self.pipeline);
        buffer.push_constant(&self.pipeline_layout, ShaderStageFlags::COMPUTE, &dispatch_params); 
        buffer.dispatch(dispatch_size.x, dispatch_size.y, dispatch_size.z);
    }
}
