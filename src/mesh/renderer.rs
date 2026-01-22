use octa_force::{camera::Camera, engine::{self, Engine}, glam::{Mat4, Vec2}, vulkan::{CommandBuffer, Context, GraphicsPipeline, GraphicsPipelineCreateInfo, GraphicsShaderCreateInfo, PipelineLayout, ash::vk::{self, AttachmentLoadOp, BlendFactor, BlendOp, ColorComponentFlags, Format, PipelineColorBlendAttachmentState, PushConstantRange, ShaderStageFlags}}};
use spirv_struct_layout::SpirvLayout;

use crate::mesh::{Mesh, Vertex, gpu_mesh::GPUMesh};


#[derive(Debug)]
pub struct MeshRenderer {
    push_constant_range: PushConstantRange,
    pipeline_layout: PipelineLayout,
    pipeline: GraphicsPipeline,
}
#[derive(Debug, Default)]
#[repr(C)]
#[derive(SpirvLayout)]
pub struct MeshDispatchParams {
    proj_mat: Mat4,
}

impl MeshRenderer {
    pub fn new(context: &Context, format: Format, depth_format: Format) -> Self {
      
        let push_constant_range = PushConstantRange::default()
            .offset(0)
            .size(size_of::<MeshDispatchParams>() as _)
            .stage_flags(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT);

        let pipeline_layout = context.create_pipeline_layout(
            &[], &[push_constant_range])
            .expect("Failed to create Pipeline Layout");

        let pipeline = context.create_graphics_pipeline::<Vertex>(
            &pipeline_layout,
            GraphicsPipelineCreateInfo {
                shaders: &[
                    GraphicsShaderCreateInfo {
                        source: &include_bytes!("../../shaders/bin/mesh_vertex.spv")[..],
                        stage: vk::ShaderStageFlags::VERTEX,
                    },
                    GraphicsShaderCreateInfo {
                        source: &include_bytes!("../../shaders/bin/mesh_fragment.spv")[..],
                        stage: vk::ShaderStageFlags::FRAGMENT,
                    },
                ],
                primitive_topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                extent: None,
                color_attachment_format: format,
                color_attachment_blend: Some(
                    PipelineColorBlendAttachmentState::default()
                        .color_write_mask(ColorComponentFlags::RGBA)
                        .blend_enable(true)
                        .src_color_blend_factor(BlendFactor::SRC_ALPHA)
                        .dst_color_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
                        .color_blend_op(BlendOp::ADD)
                        .src_alpha_blend_factor(BlendFactor::ONE)
                        .dst_alpha_blend_factor(BlendFactor::ZERO)
                        .alpha_blend_op(BlendOp::ADD),
                ),

                depth_attachment_format: depth_format,
                dynamic_states: Some(&[vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT]),
            },
        ).expect("Failed to create Pipeline");

        Self {
            push_constant_range,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn start_render(
        &self,
        buffer: &CommandBuffer,
        engine: &Engine,
        camera: &Camera,
    ) {
        buffer.begin_rendering(
            &engine.get_current_swapchain_image_and_view().view,
            &engine.get_current_depth_image_and_view().view,
            engine.swapchain.size,
            AttachmentLoadOp::DONT_CARE,
            None,
        );

        buffer.set_viewport_size(engine.swapchain.size.as_vec2());
        buffer.set_scissor_size(engine.swapchain.size.as_vec2());

        buffer.bind_graphics_pipeline(&self.pipeline);

        let proj_mat = camera.projection_matrix().mul_mat4(&camera.view_matrix());

        let dispatch_params = MeshDispatchParams {
            proj_mat,
        };

        buffer.push_constant(&self.pipeline_layout, 
            ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT, 
            &dispatch_params);
    }

    pub fn render(&self,
        buffer: &CommandBuffer,
        mesh: &GPUMesh,
    ) { 
        buffer.bind_vertex_buffer(&mesh.vertex_buffer);
        buffer.bind_index_buffer(&mesh.index_buffer);
       
        buffer.draw_indexed(mesh.index_count as _); 
    }
}
