pub mod render_data;
pub mod voxel_tree64_buffer;
pub mod palette;
pub mod g_buffer;
pub mod frame_perf_stats;
pub mod shader_stage;

use std::time::Duration;

use frame_perf_stats::FramePerfStats;
use g_buffer::{GBuffer, ImageAndViewAndHandle};
use octa_force::anyhow::Result;
use octa_force::camera::Camera;
use octa_force::egui::{Align, Frame, Layout};
use octa_force::engine::Engine;
use octa_force::glam::{uvec3, UVec2, Vec2, Vec3};
use octa_force::image::{GenericImageView, ImageReader};
use octa_force::log::info;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::{self, BufferDeviceAddressInfo, Format, PushConstantRange, ShaderStageFlags};
use octa_force::vulkan::descriptor_heap::{DescriptorHandleValue, DescriptorHeap};
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::sampler_pool::{SamplerPool, SamplerSetHandle};
use octa_force::vulkan::{
    Buffer, CommandBuffer, ComputePipeline, ComputePipelineCreateInfo, Context, DescriptorPool, DescriptorSet, DescriptorSetLayout, ImageAndView, PipelineLayout, Swapchain, WriteDescriptorSet, WriteDescriptorSetKind
};
use octa_force::{egui, in_flight_frames, OctaResult};
use palette::Palette;
use render_data::RenderData;
use shader_stage::ShaderStage;
use voxel_tree64_buffer::{VoxelTree64Buffer, VoxelTreeData};

use crate::NUM_FRAMES_IN_FLIGHT;

use super::VoxelTree64;

const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 8;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 8;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Tree64Renderer {
    heap: DescriptorHeap,
    voxel_tree64_buffer: VoxelTree64Buffer,
    palette: Palette,
    
    g_buffer: GBuffer,
    perf_stats: FramePerfStats,

    blue_noise_tex: ImageAndViewAndHandle,
   
    trace_ray_stage: ShaderStage<TraceRayDispatchParams>,
    denoise_stage: ShaderStage<DenoiseDispatchParams>,
    compose_stage: ShaderStage<ComposeDispatchParams>,

    debug_channel: DebugChannel,
    max_bounces: usize,
    heat_map_max: f32,
    temporal_denoise: bool,
    denoise_counters: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct TraceRayDispatchParams {
    tree: VoxelTreeData,
    g_buffer_ptr: u64,
    palette_ptr: u64,
    perf_stats_ptr: u64,
    max_bounces: u32,
    blue_noise_tex: DescriptorHandleValue,
}

#[repr(C)]
#[derive(Debug)]
pub struct DenoiseDispatchParams {
    g_buffer_ptr: u64,
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum DebugChannel {
    None, 
    Albedo,
    Irradiance,
    Normals,
    Depth,
    HeatMap,
    Variance,
}

#[repr(C)]
#[derive(Debug)]
pub struct ComposeDispatchParams {
    g_buffer_ptr: u64,
    debug_channel: DebugChannel,
    heat_map_range: Vec2,
}

impl Tree64Renderer {
    pub fn new(
        context: &Context,
        swapchain: &Swapchain,
        tree: VoxelTree64,
        camera: &Camera,
    ) -> Result<Tree64Renderer> {

        let mut heap = context.create_descriptor_heap(vec![
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: 30,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 30,
            },
        ])?;

        let voxel_tree64_buffer = tree.into_buffer(context)?;

        let palette = Palette::new(context)?;
 
        let g_buffer = GBuffer::new(context, &mut heap, camera, swapchain)?;

        let perf_stats = FramePerfStats::new(context)?;

        let img = ImageReader::open("assets/stbn_vec2_2Dx1D_128x128x64_combined.png")?.decode()?;
        let red_green_pixels = img.pixels()
                .map(|(_, _, p)| [p.0[0], p.0[1]])
                .flatten()
                .collect::<Vec<_>>();

        let blue_noise_tex = context.create_texture_image_from_data(
            Format::R8G8_UINT, UVec2 { x: img.width(), y: img.height() }, &red_green_pixels)?;

        let blue_noise_handle = heap.create_image_handle(
            &blue_noise_tex.view, 
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC)?;
        let blue_noise_tex = ImageAndViewAndHandle { image: blue_noise_tex.image, view: blue_noise_tex.view, handle: blue_noise_handle };    

        let sets = &[&heap.layout];
        let push_constant_size = size_of::<TraceRayDispatchParams>()
            .max(size_of::<DenoiseDispatchParams>())
            .max(size_of::<ComposeDispatchParams>()) as u32;
        
        let trace_ray_stage = ShaderStage::new(
            context, 
            include_bytes!("../../../slang_shaders/bin/trace_ray.spv"), 
            sets, push_constant_size)?;

        let denoise_stage = ShaderStage::new(
            context, 
            include_bytes!("../../../slang_shaders/bin/temporal_denoise.spv"), 
            sets, push_constant_size)?;

        let compose_stage = ShaderStage::new(
            context, 
            include_bytes!("../../../slang_shaders/bin/blit.spv"), 
            sets, push_constant_size)?;
         
        Ok(Tree64Renderer {
            heap,
            voxel_tree64_buffer,
            palette,

            g_buffer,

            perf_stats,
            blue_noise_tex,

            trace_ray_stage,
            denoise_stage,
            compose_stage,

            debug_channel: DebugChannel::None,
            max_bounces: 2,
            heat_map_max: 20.0,
            temporal_denoise: true,
            denoise_counters: true,
        })
    }

    pub fn update(&mut self, camera: &Camera, context: &Context, res: UVec2, in_flight_frame_index: usize, frame_index: usize) -> OctaResult<()> {
        self.g_buffer.update(camera, context, res, in_flight_frame_index, frame_index, self.denoise_counters)?;

        Ok(())
    }

    pub fn render(
        &self,
        buffer: &CommandBuffer,
        engine: &Engine,
    ) -> Result<()> {
        let dispatch_size = uvec3(
            (engine.get_resolution().x / RENDER_DISPATCH_GROUP_SIZE_X) + 1,
            (engine.get_resolution().y / RENDER_DISPATCH_GROUP_SIZE_Y) + 1,
            1);

        buffer.bind_descriptor_sets(
            vk::PipelineBindPoint::COMPUTE,
            &self.trace_ray_stage.pipeline_layout,
            0,
            &[&self.heap.set],
        );

        self.trace_ray_stage.render(buffer, TraceRayDispatchParams {
            tree: self.voxel_tree64_buffer.get_data(),
            g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            palette_ptr: self.palette.buffer.get_device_address(),
            perf_stats_ptr: self.perf_stats.buffer.get_device_address(),
            max_bounces: self.max_bounces as _,
            blue_noise_tex: self.blue_noise_tex.handle.value,
        }, dispatch_size);

        if self.temporal_denoise {
            self.denoise_stage.render(buffer, DenoiseDispatchParams {
                g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            }, dispatch_size);
        }

        self.compose_stage.render(buffer, ComposeDispatchParams {
            g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            debug_channel: self.debug_channel,
            heat_map_range: Vec2 { x: 0.0, y: self.heat_map_max }
        }, dispatch_size); 

        match &self.g_buffer.output_tex {
            g_buffer::OutputTexs::Storage(t) => {
                buffer.swapchain_image_copy_from_compute_storage_image(
                    &t[engine.get_current_in_flight_frame_index()].image,
                    &engine.get_current_swapchain_image_and_view().image,
                )?;
            },
            g_buffer::OutputTexs::Swapchain(..) => {
                buffer.swapchain_image_after_blit_barrier(&engine.get_current_swapchain_image_and_view().image)?;
            },
        } 
         
        Ok(())
    }

    pub fn render_ui(&mut self, ctx: &egui::Context) { 
        egui::Window::new("Debug")
            .show(ctx, |ui| {
                egui::ComboBox::from_label("Channel")
                    .selected_text(format!("{:?}", self.debug_channel))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.debug_channel, DebugChannel::None, "None");
                        ui.selectable_value(&mut self.debug_channel, DebugChannel::Albedo, "Albedo");
                        ui.selectable_value(&mut self.debug_channel, DebugChannel::Irradiance, "Irradiance");
                        ui.selectable_value(&mut self.debug_channel, DebugChannel::Depth, "Depth");
                        ui.selectable_value(&mut self.debug_channel, DebugChannel::Normals, "Normals");
                        ui.selectable_value(&mut self.debug_channel, DebugChannel::HeatMap, "HeatMap");
                        ui.selectable_value(&mut self.debug_channel, DebugChannel::Variance, "Variance");
                    }
                    );
                ui.add(egui::Slider::new(&mut self.max_bounces, 0..=10)
                    .text("Max Bounces")
                );
                ui.add(egui::Slider::new(&mut self.heat_map_max, 0.0..=256.0)
                    .text("Heat Map")
                );
                div(ui, |ui| {
                    ui.checkbox(&mut self.temporal_denoise, "Temporal Denoise");
                    ui.checkbox(&mut self.denoise_counters, "Jitter");
                });

                ui.separator();

                ui.label(format!("GBuffer: {}", self.g_buffer.frame_no -1));
                ui.label(format!("Steady : {}", self.g_buffer.num_steady_frames -1));

            });
    }

    pub fn on_recreate_swapchain(
        &mut self,
        context: &Context,
        swapchain: &Swapchain,
    ) -> Result<()> {
        self.g_buffer.on_recreate_swapchain(context, &mut self.heap, swapchain)?;

        Ok(())
    }
}

fn div(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame::none().show(ui, |ui| {
        ui.with_layout(Layout::left_to_right(Align::TOP), add_contents);
    });
}
