pub mod render_data;
pub mod voxel_tree64_buffer;
pub mod palette;

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
use palette::Palette;
use render_data::RenderData;
use voxel_tree64_buffer::{VoxelTree64Buffer, VoxelTreeData};

use super::VoxelTree64;

const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 8;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 8;


#[repr(C)]
pub struct DispatchParams {
  render_data: RenderData,
  tree: VoxelTreeData,
  palette_ptr: u64,
  max_bounces: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Tree64Renderer {
    storage_images: Vec<ImageAndView>,
    voxel_tree64_buffer: VoxelTree64Buffer,
    palette: Palette,
    
    descriptor_pool: DescriptorPool,
    descriptor_layout: DescriptorSetLayout,
    descriptor_sets: Vec<DescriptorSet>,

    push_constant_range: PushConstantRange,
    pipeline_layout: PipelineLayout,
    pipeline: ComputePipeline,
}

impl Tree64Renderer {
    pub fn new(
        context: &Context,
        res: UVec2,
        num_frames: usize,
        tree: VoxelTree64,
    ) -> Result<Tree64Renderer> {
        let storage_images = context.create_storage_images(res, num_frames)?;
    
        let voxel_tree64_buffer = tree.into_buffer(context)?;

        let palette = Palette::new(context)?;

        let descriptor_heap = context.create_descriptor_heap(vec![
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 10,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: 10,
            },
        ])?;
 
        let push_constant_range = PushConstantRange::default()
            .offset(0)
            .size(size_of::<DispatchParams>() as _)
            .stage_flags(ShaderStageFlags::COMPUTE);

        let pipeline_layout = context.create_pipeline_layout(
            &[&descriptor_heap.layout],
            &[push_constant_range])?;

        let pipeline = context.create_compute_pipeline(
            &pipeline_layout,
            ComputePipelineCreateInfo {
                shader_source: include_bytes!("../../../slang_shaders/bin/render.spv"),
            },
        )?;

        Ok(Tree64Renderer {
            storage_images,
            voxel_tree64_buffer,
            palette,

            descriptor_pool,
            descriptor_layout,
            descriptor_sets,

            push_constant_range,
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
    ) -> Result<()> {
        buffer.bind_compute_pipeline(&self.pipeline);

        buffer.bind_descriptor_sets(
            vk::PipelineBindPoint::COMPUTE,
            &self.pipeline_layout,
            0,
            &[&self.descriptor_sets[frame_index]],
        );

        let dispatch_params = DispatchParams {
            render_data: RenderData::new(cam, swapchain.size),
            tree: self.voxel_tree64_buffer.get_data(),
            palette_ptr: self.palette.buffer.get_device_address(),
            max_bounces: 0,
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

        for (i, descriotor_set) in self.descriptor_sets.iter().enumerate() {
            descriotor_set.update(&[WriteDescriptorSet {
                binding: 0,
                kind: WriteDescriptorSetKind::StorageImage {
                    layout: vk::ImageLayout::GENERAL,
                    view: &self.storage_images[i].view,
                },
            }]);
        }

        Ok(())
    }
}
