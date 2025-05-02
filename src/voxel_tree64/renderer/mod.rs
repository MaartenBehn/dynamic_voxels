pub mod render_data;
pub mod voxel_tree64_buffer;

use std::time::Duration;

use octa_force::anyhow::Result;
use octa_force::camera::Camera;
use octa_force::glam::{UVec2, Vec3};
use octa_force::log::info;
use octa_force::vulkan::ash::vk::{self, BufferDeviceAddressInfo, PushConstantRange, ShaderStageFlags};
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::{
    Buffer, CommandBuffer, ComputePipeline, ComputePipelineCreateInfo, Context, DescriptorPool, DescriptorSet, DescriptorSetLayout, ImageAndView, PipelineLayout, Swapchain, WriteDescriptorSet, WriteDescriptorSetKind
};
use render_data::RenderData;
use voxel_tree64_buffer::{VoxelTree64Buffer, VoxelTreeData};

use super::VoxelTree64;

const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 8;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 8;


#[repr(C)]
pub struct DispatchParams {
  render_data: RenderData,
  tree: VoxelTreeData,
  image_ptr: u64,
  palette_ptr: u64,
  max_bounces: u32,
}

#[allow(dead_code)]
pub struct Renderer {
    storage_images: Vec<ImageAndView>,
    voxel_tree64_buffer: VoxelTree64Buffer,

    push_constant_range: PushConstantRange,
    pipeline_layout: PipelineLayout,
    pipeline: ComputePipeline,
}

impl Renderer {
    pub fn new(
        context: &Context,
        res: UVec2,
        num_frames: usize,
        tree: VoxelTree64,
    ) -> Result<Renderer> {
        let storage_images = context.create_storage_images(res, num_frames)?;
    
        let voxel_tree64_buffer = tree.into_buffer(context)?;

        let push_constant_range = PushConstantRange::default()
            .offset(0)
            .size(size_of::<RenderData>() as _)
            .stage_flags(ShaderStageFlags::COMPUTE);

        let pipeline_layout = context.create_pipeline_layout(
            &[],
            &[push_constant_range])?;

        let pipeline = context.create_compute_pipeline(
            &pipeline_layout,
            ComputePipelineCreateInfo {
                shader_source: include_bytes!("../../../slang_shaders/bin/render.spv"),
            },
        )?;

        Ok(Renderer {
            storage_images,
            push_constant_range,

            voxel_tree64_buffer,

            pipeline_layout,
            pipeline,
        })
    }

    pub fn render(
        &self,
        buffer: &CommandBuffer,
        frame_index: usize,
        swapchain: &Swapchain,
        cam: &Camera,
        res: UVec2,
    ) -> Result<()> {
        buffer.bind_compute_pipeline(&self.pipeline);

        let dispatch_params = DispatchParams {
            render_data: RenderData::new(cam, res),
            tree: self.voxel_tree64_buffer.get_data(),
            image_ptr: self.

        };
      
        buffer.push_constant(&self.pipeline_layout, ShaderStageFlags::COMPUTE, &dispatch_params);
        buffer.dispatch(
            (swapchain.size.x / RENDER_DISPATCH_GROUP_SIZE_X) + 1,
            (swapchain.size.y / RENDER_DISPATCH_GROUP_SIZE_Y) + 1,
            1,
        );

        buffer.swapchain_image_copy_from_compute_storage_image(
            &self.storage_images[frame_index].image,
            &swapchain.images_and_views[frame_index].image,
        )?;

        Ok(())
    }

    pub fn on_recreate_swapchain(
        &mut self,
        context: &Context,
        num_frames: usize,
        res: UVec2,
    ) -> Result<()> {
        self.storage_images = context.create_storage_images(res, num_frames)?;

        Ok(())
    }
}
