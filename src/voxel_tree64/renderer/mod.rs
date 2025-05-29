pub mod render_data;
pub mod voxel_tree64_buffer;
pub mod palette;
pub mod g_buffer;
pub mod frame_perf_stats;

use std::time::Duration;

use frame_perf_stats::FramePerfStats;
use g_buffer::{GBuffer, ImageAndViewAndHandle};
use octa_force::anyhow::Result;
use octa_force::camera::Camera;
use octa_force::glam::{UVec2, Vec3};
use octa_force::image::ImageReader;
use octa_force::log::info;
use octa_force::vulkan::ash::vk::{self, BufferDeviceAddressInfo, Format, PushConstantRange, ShaderStageFlags};
use octa_force::vulkan::descriptor_heap::DescriptorHeap;
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::sampler_pool::{SamplerPool, SamplerSetHandle};
use octa_force::vulkan::{
    Buffer, CommandBuffer, ComputePipeline, ComputePipelineCreateInfo, Context, DescriptorPool, DescriptorSet, DescriptorSetLayout, ImageAndView, PipelineLayout, Swapchain, WriteDescriptorSet, WriteDescriptorSetKind
};
use octa_force::{in_flight_frames, OctaResult};
use palette::Palette;
use render_data::RenderData;
use voxel_tree64_buffer::{VoxelTree64Buffer, VoxelTreeData};

use crate::NUM_FRAMES_IN_FLIGHT;

use super::VoxelTree64;

const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 8;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 8;


#[repr(C)]
pub struct DispatchParams {
    tree: VoxelTreeData,
    g_buffer_ptr: u64,
    palette_ptr: u64,
    perf_stats_ptr: u64,
    max_bounces: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Tree64Renderer {
    voxel_tree64_buffer: VoxelTree64Buffer,
    palette: Palette,
    
    g_buffer: GBuffer,

    descriptor_heap: DescriptorHeap,
    sampler_pool: SamplerPool,
    sampler_set_handle: SamplerSetHandle,

    push_constant_range: PushConstantRange,
    pipeline_layout: PipelineLayout,
    pipeline: ComputePipeline,

    perf_stats: FramePerfStats,

    blue_noise_tex: ImageAndViewAndHandle,
}

impl Tree64Renderer {
    pub fn new(
        context: &Context,
        res: UVec2,
        tree: VoxelTree64,
        camera: &Camera,
    ) -> Result<Tree64Renderer> {

        let voxel_tree64_buffer = tree.into_buffer(context)?;

        let palette = Palette::new(context)?;

        let mut descriptor_heap = context.create_descriptor_heap(vec![
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: 20,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 20,
            },
        ])?;

        let g_buffer = GBuffer::new(context, res, &mut descriptor_heap, camera)?;

        let perf_stats = FramePerfStats::new(context)?;

        let img = ImageReader::open("assets/stbn_vec2_2Dx1D_128x128x64_combined.png")?.decode()?;
        let blue_noise_tex = context.create_texture_image_from_data(
            Format::R8G8_UINT, UVec2 { x: img.width(), y: img.height() }, img.as_bytes())?;

        let blue_noise_handle = descriptor_heap.create_image_handle(
            &blue_noise_tex.view, 
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC)?;
        let blue_noise_tex = ImageAndViewAndHandle { image: blue_noise_tex.image, view: blue_noise_tex.view, handle: blue_noise_handle };

        let mut sampler_pool = context.create_sampler_pool(1)?; 
        let sampler_set_handle = sampler_pool.get_set(
            &[
                vk::SamplerCreateInfo::default()
                    .mag_filter(vk::Filter::LINEAR)
                    .min_filter(vk::Filter::LINEAR)
                    .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                    .min_lod(0.0)
                    .max_lod(vk::LOD_CLAMP_NONE)
            ]
        )?;

        let push_constant_range = PushConstantRange::default()
            .offset(0)
            .size(size_of::<DispatchParams>() as _)
            .stage_flags(ShaderStageFlags::COMPUTE);

        let pipeline_layout = context.create_pipeline_layout(
            &[&descriptor_heap.layout, &sampler_set_handle.layout],
            &[push_constant_range])?;

        let pipeline = context.create_compute_pipeline(
            &pipeline_layout,
            ComputePipelineCreateInfo {
                shader_source: include_bytes!("../../../slang_shaders/bin/render.spv"),
            },
        )?;

        Ok(Tree64Renderer {
            voxel_tree64_buffer,
            palette,

            g_buffer,

            descriptor_heap,
            sampler_pool,
            sampler_set_handle,

            push_constant_range,
            pipeline_layout,
            pipeline,

            perf_stats,
            blue_noise_tex,
        })
    }

    pub fn update(&mut self, current_index: usize, frame_no: usize, camera: &Camera) -> OctaResult<()> {
        self.g_buffer.push_uniform(current_index, frame_no, camera)?;

        Ok(())
    }

    pub fn render(
        &self,
        buffer: &CommandBuffer, 
        swapchain: &Swapchain,
        in_flight_frames_index: usize,
    ) -> Result<()> {
        buffer.bind_descriptor_sets(
            vk::PipelineBindPoint::COMPUTE,
            &self.pipeline_layout,
            0,
            &[&self.descriptor_heap.set, &self.sampler_set_handle.set],
        );

        buffer.bind_compute_pipeline(&self.pipeline);
        
        let dispatch_params = DispatchParams {
            tree: self.voxel_tree64_buffer.get_data(),
            g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            palette_ptr: self.palette.buffer.get_device_address(),
            perf_stats_ptr: self.perf_stats.buffer.get_device_address(),
            max_bounces: 0,
        };

        buffer.push_constant(&self.pipeline_layout, ShaderStageFlags::COMPUTE, &dispatch_params);
        buffer.dispatch(
            (swapchain.size.x / RENDER_DISPATCH_GROUP_SIZE_X) + 1,
            (swapchain.size.y / RENDER_DISPATCH_GROUP_SIZE_Y) + 1,
            1,
        );

        buffer.swapchain_image_copy_from_compute_storage_image(
            &self.g_buffer.albedo_tex[in_flight_frames_index].image,
            &swapchain.images_and_views[swapchain.current_index].image,
        )?;

        Ok(())
    }

    pub fn on_recreate_swapchain(
        &mut self,
        context: &Context,
        num_frames: usize,
        res: UVec2,
    ) -> Result<()> {
        self.g_buffer.on_recreate_swapchain(context, res, &mut self.descriptor_heap)?;

        Ok(())
    }
}
