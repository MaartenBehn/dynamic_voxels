pub mod render_data;
pub mod g_buffer;
pub mod shader_stage;

use std::mem;
use std::time::Duration;

use egui_double_slider::DoubleSlider;
use g_buffer::{GBuffer, ImageAndViewAndHandle};
use octa_force::anyhow::Result;
use octa_force::camera::Camera;
use octa_force::egui::{Align, Frame, Layout};
use octa_force::engine::Engine;
use octa_force::glam::{uvec3, vec2, UVec2, Vec2, Vec3};
use octa_force::image::{GenericImageView, ImageReader};
use octa_force::log::{debug, info};
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::{self, BufferDeviceAddressInfo, Format, PushConstantRange, ShaderStageFlags};
use octa_force::vulkan::descriptor_heap::{DescriptorHandleValue, ImageDescriptorHeap};
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::sampler_pool::{SamplerPool, SamplerSetHandle};
use octa_force::vulkan::{
    Buffer, CommandBuffer, ComputePipeline, ComputePipelineCreateInfo, Context, DescriptorPool, DescriptorSet, DescriptorSetLayout, ImageAndView, PipelineLayout, Swapchain, WriteDescriptorSet, WriteDescriptorSetKind
};
use octa_force::{egui, in_flight_frames, OctaResult};
use render_data::RenderData;
use shader_stage::ShaderStage;
use spirv_struct_layout::{CheckSpirvStruct, SpirvLayout};
use super::palette::buffer::PaletteBuffer;
use super::palette::shared::SharedPalette;

use crate::NUM_FRAMES_IN_FLIGHT;


const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 8;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 8;

#[allow(dead_code)]
#[derive(Debug)]
pub struct VoxelRenderer {
    heap: ImageDescriptorHeap,
    palette_buffer: PaletteBuffer,
    
    g_buffer: GBuffer,

    blue_noise_tex: ImageAndViewAndHandle,
   
    trace_ray_stage: ShaderStage,
    denoise_stage: ShaderStage,
    filter_stage: ShaderStage,
    blit_stage: ShaderStage,

    pub debug_channel: DebugChannel,
    pub max_bounces: usize,
    debug_depth_range: Vec2,
    debug_heat_map_range: Vec2,
    pub temporal_denoise: bool,
    pub denoise_counters: bool,
    static_accum_number: u32,

    filter_passes: usize,
    temp_irradiance_tex: ImageAndViewAndHandle,

    render_into_swapchain: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct RayManagerData {
    g_buffer_ptr: u64,
    palette_ptr: u64,
    max_bounces: u32,
    blue_noise_tex: DescriptorHandleValue,
}

#[repr(C)]
#[derive(Debug)]
pub struct TemporalDenoiseDispatchParams {
    g_buffer_ptr: u64,
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

#[repr(C)]
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
#[derive(SpirvLayout)]
#[derive(Debug)]
pub struct ComposeDispatchParams {
    g_buffer_ptr: u64,
    debug_depth_range: Vec2,
    debug_heat_map_range: Vec2,
    debug_channel: DebugChannel,
}

impl VoxelRenderer {
    pub fn new<D>(
        context: &Context,
        swapchain: &Swapchain,
        camera: &Camera,
        palette: SharedPalette,
        trace_ray_bin: &[u8],
        allways_fullscreen: bool,
    ) -> OctaResult<VoxelRenderer> {

        let render_into_swapchain = allways_fullscreen && context.swapchain_supports_storage();

        let mut heap = context.create_descriptor_heap(40)?;

        let palette_buffer = PaletteBuffer::new(context, palette)?;

        let g_buffer = GBuffer::new(context, &mut heap, camera, swapchain.size, render_into_swapchain, swapchain)?;

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
            swapchain.size)?;

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
            include_bytes!("../../../shaders/slang/bin/_temporal_denoise.spv"), 
            sets, push_constant_size)?;

        let filter_stage = ShaderStage::new(
            context, 
            include_bytes!("../../../shaders/slang/bin/_a_tours_filter.spv"), 
            sets, push_constant_size)?;

        let blit_stage = ShaderStage::new(
            context, 
            include_bytes!("../../../shaders/slang/bin/_blit.spv"), 
            sets, push_constant_size)?;
         
        Ok(VoxelRenderer {
            heap,
            palette_buffer,

            g_buffer,

            blue_noise_tex,

            trace_ray_stage,
            denoise_stage,
            filter_stage,
            blit_stage,

            debug_channel: DebugChannel::None,
            max_bounces: 1,
            temporal_denoise: true,
            denoise_counters: true,
            filter_passes: 2,
            temp_irradiance_tex,
            static_accum_number: 20,

            debug_depth_range: vec2(0.5, 1.1),
            debug_heat_map_range: vec2(0.0, 100.0),

            render_into_swapchain,
        })
    }

    pub fn get_ray_manager_data(&self) -> RayManagerData {
        RayManagerData {
            g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            palette_ptr: self.palette_buffer.buffer.get_device_address(),
            max_bounces: self.max_bounces as _,
            blue_noise_tex: self.blue_noise_tex.handle.value,
        }
    }

    pub fn update(&mut self, camera: &Camera, context: &Context, size: UVec2, in_flight_frame_index: usize, frame_index: usize) -> OctaResult<()> {
        self.g_buffer.update(camera, context, size, in_flight_frame_index, frame_index, self.denoise_counters)?;
        self.palette_buffer.update(context)?;

        Ok(())
    }

    pub fn render<D>(
        &mut self,
        offset: UVec2,
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

        if self.temporal_denoise && self.debug_channel != DebugChannel::HeatMap {
            self.denoise_stage.render(buffer, TemporalDenoiseDispatchParams {
                g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
                static_accum_number: self.static_accum_number,
            } ,dispatch_size);

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
        }
 
        self.blit_stage.render(buffer, ComposeDispatchParams {
            g_buffer_ptr: self.g_buffer.uniform_buffer.get_device_address(),
            debug_channel: self.debug_channel,
            debug_depth_range: self.debug_depth_range,
            debug_heat_map_range: self.debug_heat_map_range,
        }, dispatch_size);

        match &self.g_buffer.output_tex {
            g_buffer::OutputTexs::Storage(t) => {
                buffer.swapchain_image_copy_from_compute_storage_image(
                    &t[engine.get_current_in_flight_frame_index()].image,
                    &engine.get_current_swapchain_image_and_view().image,
                    offset,
                )?;
            },
            g_buffer::OutputTexs::Swapchain(..) => {
                buffer.swapchain_image_after_blit_barrier(&engine.get_current_swapchain_image_and_view().image)?;
            },
        } 
         
        Ok(())
    }

    pub fn render_ui(&mut self, ctx: &egui::Context) { 
        egui::Window::new("Renderer")
            .default_open(false)
            .show(ctx, |ui| {
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

        egui::Window::new("Debug")
            .default_open(false)
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

                if matches!(self.debug_channel, DebugChannel::Depth) || matches!(self.debug_channel, DebugChannel::All) {  
                    ui.label(format!("Depth Range: {:.02} - {:.02}", self.debug_depth_range.x, self.debug_depth_range.y));
                    ui.add(DoubleSlider::new(
                        &mut self.debug_depth_range.x,
                        &mut self.debug_depth_range.y,
                        0.0..=10.0,
                    )
                        .width(300.0)
                        .separation_distance(0.1)
                    );
                }

                if matches!(self.debug_channel, DebugChannel::HeatMap) || matches!(self.debug_channel, DebugChannel::All) {  
                    ui.label(format!("Heat Map Range: {:.02} - {:.02}", self.debug_heat_map_range.x, self.debug_heat_map_range.y));
                    ui.add(DoubleSlider::new(
                        &mut self.debug_heat_map_range.x,
                        &mut self.debug_heat_map_range.y,
                        0.0..=255.0,
                    )
                        .width(300.0)
                        .separation_distance(0.0)
                    );
                }
            });

    }

    pub fn on_size_changed(
        &mut self,
        context: &Context,
        size: UVec2,
        swapchain: &Swapchain,
    ) -> OctaResult<()> {
        self.g_buffer.on_size_changed(context, &mut self.heap, size, self.render_into_swapchain, swapchain)?;

        let temp_irradiance_image = context.create_image(
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC, 
            MemoryLocation::GpuOnly, 
            Format::R8G8B8A8_UNORM, 
            size)?;

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
    Frame::NONE.show(ui, |ui| {
        ui.with_layout(Layout::left_to_right(Align::TOP), add_contents);
    });
}
