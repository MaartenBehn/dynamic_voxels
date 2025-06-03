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
use octa_force::engine::Engine;
use octa_force::glam::{UVec2, Vec2, Vec3};
use octa_force::image::ImageReader;
use octa_force::log::info;
use octa_force::vulkan::ash::vk::{self, BufferDeviceAddressInfo, Format, PushConstantRange, ShaderStageFlags};
use octa_force::vulkan::descriptor_heap::{DescriptorHandleValue, DescriptorHeap};
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct Tree64Renderer {
    voxel_tree64_buffer: VoxelTree64Buffer,
    palette: Palette,
    
    g_buffer: GBuffer,

    render_push_constant_range: PushConstantRange,
    render_pipeline_layout: PipelineLayout,
    render_pipeline: ComputePipeline,

    compose_push_constant_range: PushConstantRange,
    compose_pipeline_layout: PipelineLayout,
    compose_pipeline: ComputePipeline,

    perf_stats: FramePerfStats,

    blue_noise_tex: ImageAndView,
    pub descriptor_pool: DescriptorPool,
    pub descriptor_layout: DescriptorSetLayout,
    pub descriptor_set: DescriptorSet, 
}

#[repr(C)]
pub struct RenderDispatchParams {
    tree: VoxelTreeData,
    g_buffer_ptr: u64,
    palette_ptr: u64,
    perf_stats_ptr: u64,
    max_bounces: u32,
}

#[repr(u32)]
pub enum DebugChannel {
    None, 
    Albedo,
    Irradiance,
    Normals,
    HeatMap,
    Variance,
}

#[repr(C)]
pub struct ComposeDispatchParams {
    g_buffer_ptr: u64,
    debug_channel: DebugChannel,
    heat_map_range: Vec2,
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
 
        let g_buffer = GBuffer::new(context, res, camera)?;

        let perf_stats = FramePerfStats::new(context)?;

        let img = ImageReader::open("assets/stbn_vec2_2Dx1D_128x128x64_combined.png")?.decode()?;
        let blue_noise_tex = context.create_texture_image_from_data(
            Format::R8G8_UINT, UVec2 { x: img.width(), y: img.height() }, img.as_bytes())?;

        let descriptor_pool = context.create_descriptor_pool(
            1,
            &[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLED_IMAGE,
                    descriptor_count: 1,
                },
            ],
        )?;

        let descriptor_layout_bindings = vec![vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::SAMPLED_IMAGE,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            }
        ];

        let descriptor_layout =
            context.create_descriptor_set_layout(&descriptor_layout_bindings)?;

        let descriptor_set = descriptor_pool.allocate_set(&descriptor_layout)?;
        let mut write_descriptor_sets = vec![WriteDescriptorSet {
            binding: 0,
            kind: WriteDescriptorSetKind::SampledImage {
                layout: vk::ImageLayout::GENERAL,
                view: &blue_noise_tex.view,
            },
        }];

        descriptor_set.update(&write_descriptor_sets); 
        
        let render_push_constant_range = PushConstantRange::default()
            .offset(0)
            .size(size_of::<RenderDispatchParams>() as _)
            .stage_flags(ShaderStageFlags::COMPUTE);

        let render_pipeline_layout = context.create_pipeline_layout(
            &[&g_buffer.descriptor_layout, &descriptor_layout],
            &[render_push_constant_range])?;

        let render_pipeline = context.create_compute_pipeline(
            &render_pipeline_layout,
            ComputePipelineCreateInfo {
                shader_source: include_bytes!("../../../slang_shaders/bin/render.spv"),
            },
        )?;


        let compose_push_constant_range = PushConstantRange::default()
            .offset(0)
            .size(size_of::<RenderDispatchParams>() as _)
            .stage_flags(ShaderStageFlags::COMPUTE);

        let compose_pipeline_layout = context.create_pipeline_layout(
            &[&g_buffer.descriptor_layout],
            &[compose_push_constant_range])?;

        let compose_pipeline = context.create_compute_pipeline(
            &compose_pipeline_layout,
            ComputePipelineCreateInfo {
                shader_source: include_bytes!("../../../slang_shaders/bin/compose.spv"),
            },
        )?;

        Ok(Tree64Renderer {
            voxel_tree64_buffer,
            palette,

            g_buffer,

            render_push_constant_range,
            render_pipeline_layout,
            render_pipeline,

            compose_push_constant_range,
            compose_pipeline_layout,
            compose_pipeline,

            perf_stats,
            blue_noise_tex,
            descriptor_pool,
            descriptor_layout,
            descriptor_set
        })
    }

    pub fn update(&mut self, frame_no: usize, camera: &Camera, context: &Context, res: UVec2) -> OctaResult<()> {
        self.g_buffer.push_uniform(frame_no, camera, context, res)?;

        Ok(())
    }

    pub fn render(
        &self,
        buffer: &CommandBuffer,
        engine: &Engine,
    ) -> Result<()> {
        buffer.bind_descriptor_sets(
            vk::PipelineBindPoint::COMPUTE,
            &self.render_pipeline_layout,
            0,
            &[&self.g_buffer.descriptor_sets[engine.get_current_in_flight_frame_index()], &self.descriptor_set],
        );

        buffer.bind_compute_pipeline(&self.render_pipeline);
        
        let dispatch_params = RenderDispatchParams {
            tree: self.voxel_tree64_buffer.get_data(),
            g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            palette_ptr: self.palette.buffer.get_device_address(),
            perf_stats_ptr: self.perf_stats.buffer.get_device_address(),
            max_bounces: 0,
        };

        buffer.push_constant(&self.render_pipeline_layout, ShaderStageFlags::COMPUTE, &dispatch_params); 

        buffer.dispatch(
            (engine.get_resolution().x / RENDER_DISPATCH_GROUP_SIZE_X) + 1,
            (engine.get_resolution().y / RENDER_DISPATCH_GROUP_SIZE_Y) + 1,
            1,
        );


        buffer.bind_compute_pipeline(&self.compose_pipeline);

        let dispatch_params = ComposeDispatchParams {
            g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            debug_channel: DebugChannel::None,
            heat_map_range: Vec2 { x: 0.0, y: 256.0 }
        };

        buffer.push_constant(&self.compose_pipeline_layout, ShaderStageFlags::COMPUTE, &dispatch_params); 

        buffer.dispatch(
            (engine.get_resolution().x / RENDER_DISPATCH_GROUP_SIZE_X) + 1,
            (engine.get_resolution().y / RENDER_DISPATCH_GROUP_SIZE_Y) + 1,
            1,
        );


        buffer.swapchain_image_copy_from_compute_storage_image(
            &self.g_buffer.albedo_tex[engine.get_current_in_flight_frame_index()].image,
            &engine.get_current_swapchain_image_and_view().image,
        )?;
 
        Ok(())
    }

    pub fn on_recreate_swapchain(
        &mut self,
        context: &Context,
        num_frames: usize,
        res: UVec2,
    ) -> Result<()> {
        self.g_buffer.on_recreate_swapchain(context, res)?;

        Ok(())
    }
}
