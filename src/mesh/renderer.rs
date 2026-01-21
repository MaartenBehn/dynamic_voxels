use octa_force::vulkan::{Context, GraphicsPipelineCreateInfo, GraphicsShaderCreateInfo, ash::vk::{self, Format}};

use crate::mesh::Vertex;


pub struct MeshRenderer {

}

impl MeshRenderer {
    pub fn new(context: &Context, format: Format, depth_format: Format) -> Self {
       
        let pipeline_layout = context.create_pipeline_layout(&[], &[])?;

        context.create_graphics_pipeline::<Vertex>(
            &pipeline_layout,
            GraphicsPipelineCreateInfo {
                shaders: &[
                    GraphicsShaderCreateInfo {
                        source: &include_bytes!("../shaders/shader.vert.spv")[..],
                        stage: vk::ShaderStageFlags::VERTEX,
                    },
                    GraphicsShaderCreateInfo {
                        source: &include_bytes!("../shaders/shader.frag.spv")[..],
                        stage: vk::ShaderStageFlags::FRAGMENT,
                    },
                ],
                primitive_topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                extent: None,
                color_attachment_format: format,
                color_attachment_blend: None,
                depth_attachment_format: depth_format,
                dynamic_states: Some(&[vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT]),
            },
        );

        Self {  

        }
    }
}
