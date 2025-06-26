pub mod render_data;
pub mod palette;
pub mod g_buffer;
pub mod frame_perf_stats;
pub mod shader_stage;

use std::mem;
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

use crate::NUM_FRAMES_IN_FLIGHT;

const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 8;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 8;

#[allow(dead_code)]
#[derive(Debug)]
pub struct VoxelRenderer {
    heap: DescriptorHeap,
    palette: Palette,
    
    g_buffer: GBuffer,
    perf_stats: FramePerfStats,

    blue_noise_tex: ImageAndViewAndHandle,
   
    trace_ray_stage: ShaderStage,
    denoise_stage: ShaderStage,
    filter_stage: ShaderStage,
    blit_stage: ShaderStage,

    pub debug_channel: DebugChannel,
    pub max_bounces: usize,
    heat_map_max: f32,
    pub temporal_denoise: bool,
    pub denoise_counters: bool,
    pub plane_dist: f32,
    static_accum_number: u32,

    filter_passes: usize,
    temp_irradiance_tex: ImageAndViewAndHandle, 
}

#[repr(C)]
#[derive(Debug)]
pub struct RayManagerData {
    g_buffer_ptr: u64,
    palette_ptr: u64,
    perf_stats_ptr: u64,
    max_bounces: u32,
    blue_noise_tex: DescriptorHandleValue,
}

#[repr(C)]
#[derive(Debug)]
pub struct TemporalDenoiseDispatchParams {
    g_buffer_ptr: u64,
    plane_dist: f32,
    static_accum_number: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct AToursFilterDispatchParams {
    g_buffer_ptr: u64,
    in_irradiance_tex: DescriptorHandleValue,
    out_irradiance_tex: DescriptorHandleValue,
    pass_index: u32,
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
    All,
}

#[repr(C)]
#[derive(Debug)]
pub struct ComposeDispatchParams {
    g_buffer_ptr: u64,
    debug_channel: DebugChannel,
    heat_map_range: Vec2,
}

impl VoxelRenderer {
    pub fn new<D>(
        context: &Context,
        swapchain: &Swapchain,
        camera: &Camera,
        trace_ray_bin: &[u8]
    ) -> OctaResult<VoxelRenderer> {

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


        let temp_irradiance_image = context.create_image(
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC, 
            MemoryLocation::GpuOnly, 
            Format::R8G8B8A8_UNORM, 
            swapchain.size.x, swapchain.size.y)?;

        let temp_irradiance_view = temp_irradiance_image.create_image_view(false)?;

        let temp_irradiance_handle = heap.create_image_handle(&temp_irradiance_view, vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC)?;

        let temp_irradiance_tex = ImageAndViewAndHandle {
            image: temp_irradiance_image,
            view: temp_irradiance_view,
            handle: temp_irradiance_handle,
        };

        let sets = &[&heap.layout];
        let push_constant_size = size_of::<D>()
            .max(size_of::<TemporalDenoiseDispatchParams>())
            .max(size_of::<ComposeDispatchParams>()) as u32;
        
        let trace_ray_stage = ShaderStage::new(
            context, 
            trace_ray_bin, 
            sets, push_constant_size)?;

        let denoise_stage = ShaderStage::new(
            context, 
            include_bytes!("../../slang_shaders/bin/_temporal_denoise.spv"), 
            sets, push_constant_size)?;

        let filter_stage = ShaderStage::new(
            context, 
            include_bytes!("../../slang_shaders/bin/_a_tours_filter.spv"), 
            sets, push_constant_size)?;

        let blit_stage = ShaderStage::new(
            context, 
            include_bytes!("../../slang_shaders/bin/_blit.spv"), 
            sets, push_constant_size)?;
         
        Ok(VoxelRenderer {
            heap,
            palette,

            g_buffer,

            perf_stats,
            blue_noise_tex,

            trace_ray_stage,
            denoise_stage,
            filter_stage,
            blit_stage,

            debug_channel: DebugChannel::None,
            max_bounces: 2,
            heat_map_max: 20.0,
            temporal_denoise: true,
            denoise_counters: true,
            filter_passes: 2,
            temp_irradiance_tex,
            plane_dist: 0.020,
            static_accum_number: 20,
        })
    }

    pub fn get_ray_manager_data(&self) -> RayManagerData {
        RayManagerData {
            g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            palette_ptr: self.palette.buffer.get_device_address(),
            perf_stats_ptr: self.perf_stats.buffer.get_device_address(),
            max_bounces: self.max_bounces as _,
            blue_noise_tex: self.blue_noise_tex.handle.value,
        }
    }

    pub fn update(&mut self, camera: &Camera, context: &Context, res: UVec2, in_flight_frame_index: usize, frame_index: usize) -> OctaResult<()> {
        self.g_buffer.update(camera, context, res, in_flight_frame_index, frame_index, self.denoise_counters)?;

        Ok(())
    }

    pub fn render<D>(
        &mut self,
        buffer: &CommandBuffer,
        engine: &Engine,
        trace_ray_dispact_params: D
    ) -> OctaResult<()> {
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

        self.trace_ray_stage.render(buffer, trace_ray_dispact_params, dispatch_size);

        if self.temporal_denoise {
            self.denoise_stage.render(buffer, TemporalDenoiseDispatchParams {
                g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
                plane_dist: self.plane_dist,
                static_accum_number: self.static_accum_number,
            } ,dispatch_size);
        }

        for i in 0..self.filter_passes {

            let input_tex = if i % 2 == 0 {
                self.g_buffer.irradiance_tex[engine.get_current_in_flight_frame_index()].handle.value
            } else {
                self.temp_irradiance_tex.handle.value 
            };

            let output_tex = if i % 2 == 0 {
                self.temp_irradiance_tex.handle.value 
            } else {
                self.g_buffer.irradiance_tex[engine.get_current_in_flight_frame_index()].handle.value
            };

            self.filter_stage.render(buffer, AToursFilterDispatchParams {
                g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
                in_irradiance_tex: input_tex,
                out_irradiance_tex: output_tex,
                pass_index: i as _,
            } ,dispatch_size);
        }
 
        self.blit_stage.render(buffer, ComposeDispatchParams {
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
                        ui.selectable_value(&mut self.debug_channel, DebugChannel::All, "All");
                    }
                    );
                ui.add(egui::Slider::new(&mut self.heat_map_max, 0.0..=256.0)
                    .text("Heat Map")
                );
                ui.add(egui::Slider::new(&mut self.max_bounces, 0..=10)
                    .text("Max Bounces")
                );
                ui.separator();

                div(ui, |ui| {
                    ui.checkbox(&mut self.temporal_denoise, "Temporal Denoise");
                    ui.checkbox(&mut self.denoise_counters, "Jitter");
                });

                ui.separator();
                
                ui.label(format!("Frame: {}", self.g_buffer.frame_no -1));
                ui.label(format!("Steady : {}", self.g_buffer.num_steady_frames -1));
                
                ui.add(egui::Slider::new(&mut self.plane_dist, 0.001..=0.1)
                    .logarithmic(true)
                    .text("Plane Dist")
                );

                ui.add(egui::Slider::new(&mut self.static_accum_number, 10..=255)
                    .text("Static Accum Number")
                );
                
                ui.separator();

                ui.add(egui::Slider::new(&mut self.filter_passes, 0..=10)
                    .text("Filter Passes")
                );
                if self.filter_passes % 2 == 1 {
                    self.filter_passes += 1;
                } 

            });
    }

    pub fn on_recreate_swapchain(
        &mut self,
        context: &Context,
        swapchain: &Swapchain,
    ) -> OctaResult<()> {
        self.g_buffer.on_recreate_swapchain(context, &mut self.heap, swapchain)?;

        let temp_irradiance_image = context.create_image(
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC, 
            MemoryLocation::GpuOnly, 
            Format::R8G8B8A8_UNORM, 
            swapchain.size.x, swapchain.size.y)?;

        let temp_irradiance_view = temp_irradiance_image.create_image_view(false)?;

        let temp_irradiance_handle = self.heap.create_image_handle(&temp_irradiance_view, vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC)?;

        self.temp_irradiance_tex = ImageAndViewAndHandle {
            image: temp_irradiance_image,
            view: temp_irradiance_view,
            handle: temp_irradiance_handle,
        };

        Ok(())
    }
}

fn div(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame::none().show(ui, |ui| {
        ui.with_layout(Layout::left_to_right(Align::TOP), add_contents);
    });
}
