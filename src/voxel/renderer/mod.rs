pub mod render_data;
pub mod g_buffer;
pub mod shader_stage;
pub mod temporal_denoise;
pub mod gi;
pub mod base;
//pub mod tree64_render;

use std::mem;
use std::time::Duration;

use egui_double_slider::DoubleSlider;
use g_buffer::{GBuffer, ImageAndViewAndHandle};
use octa_force::anyhow::Result;
use octa_force::camera::Camera;
use octa_force::egui::{Align, Frame, Layout, RichText};
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
use crate::voxel::renderer::base::{BaseRenderer, BlitDispatchParams, DebugChannel, SceneDispatchDispatchParams};
use crate::voxel::renderer::gi::{GIProbeUpdateData, GIRenderer};
use crate::voxel::renderer::temporal_denoise::{AToursFilterDispatchParams, TemporalDenoiseDispatchParams, TemporalDenoiseRenderer};

const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 8;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 8;

#[allow(dead_code)]
#[derive(Debug)]
pub struct VoxelRenderer {
    pub heap: ImageDescriptorHeap,
    pub palette_buffer: PaletteBuffer,
    
    pub g_buffer: GBuffer,
    pub base: BaseRenderer,
    pub gi: GIRenderer,
    pub temporal_denoise: TemporalDenoiseRenderer,
}

impl VoxelRenderer {
    pub fn new(
        context: &Context,
        swapchain: &Swapchain,
        camera: &Camera,
        palette: SharedPalette,
        allways_fullscreen: bool,
    ) -> OctaResult<VoxelRenderer> {

        let render_into_swapchain = allways_fullscreen && context.swapchain_supports_storage();

        let mut heap = context.create_descriptor_heap(40)?;
        let palette_buffer = PaletteBuffer::new(context, palette)?;
        let g_buffer = GBuffer::new(context, &mut heap, camera, swapchain.size, render_into_swapchain, swapchain)?;
        
        let push_constant_size = size_of::<SceneDispatchDispatchParams>()
            .max(size_of::<GIProbeUpdateData>())
            .max(size_of::<TemporalDenoiseDispatchParams>())
            .max(size_of::<BlitDispatchParams>()) as u32;

        let base = BaseRenderer::new(context, &mut heap, push_constant_size, render_into_swapchain)?;
        let gi = GIRenderer::new(context, &mut heap, push_constant_size)?;
        let temporal_denoise = TemporalDenoiseRenderer::new(context, swapchain, &mut heap, push_constant_size)?;

        Ok(VoxelRenderer {
            heap,
            palette_buffer,
            g_buffer,
            base,
            gi,
            temporal_denoise,
        })
    }
    
    pub fn update(&mut self, camera: &Camera, context: &Context, size: UVec2, in_flight_frame_index: usize, frame_index: usize) 
        -> OctaResult<()> {
        self.g_buffer.update(camera, context, size, in_flight_frame_index, frame_index, self.temporal_denoise.denoise_counters)?;
        self.palette_buffer.update(context)?;

        Ok(())
    }

    pub fn render(
        &mut self,
        offset: UVec2,
        buffer: &CommandBuffer,
        engine: &Engine,
    ) -> OctaResult<()> {
        let dispatch_size = uvec3(
            (engine.get_resolution().x / RENDER_DISPATCH_GROUP_SIZE_X) + 1,
            (engine.get_resolution().y / RENDER_DISPATCH_GROUP_SIZE_Y) + 1,
            1);

        buffer.bind_descriptor_sets(
            vk::PipelineBindPoint::COMPUTE,
            &self.base.trace_scene_stage.pipeline_layout,
            0,
            &[&self.heap.set],
        );

        self.base.trace_scene_stage.render(buffer, SceneDispatchDispatchParams {
            g_buffer_ptr: self.g_buffer.ptr,
            palette_ptr: self.palette_buffer.ptr,
            blue_noise_tex: self.base.blue_noise_tex.handle.value,
            start_ptr: self.base.start_ptr,
            bvh_offset: self.base.bvh_offset,
            bvh_len: self.base.bvh_len,
            active_probe_map_offset: self.gi.active_probe_map_offset,
            max_bounces: self.base.max_bounces,
        }, dispatch_size);

        if self.gi.active && self.gi.num_active_probes > 0 {
            self.gi.gi_probe_update_stage.render(buffer, GIProbeUpdateData {
                radiance_atlas: self.gi.radiance_atlas.handle.value,
                depth_atlas: self.gi.depth_atlas.handle.value,
                palette: self.palette_buffer.ptr,
                start_ptr: self.base.start_ptr,
                active_probe_data_offset: self.gi.active_probe_data_offset,
            }, uvec3(self.gi.num_active_probes, 1, 1)); //uvec3(self.gi.num_active_probes, 1, 1));
        }

        if self.temporal_denoise.temporal_denoise && self.base.debug_channel != DebugChannel::HeatMap {
            self.temporal_denoise.denoise_stage.render(buffer, TemporalDenoiseDispatchParams {
                g_buffer_ptr: self.g_buffer.ptr,
                static_accum_number: self.temporal_denoise.static_accum_number,
            } ,dispatch_size);

            for i in 0..self.temporal_denoise.filter_passes {

                let input_tex = if i % 2 == 0 {
                    self.g_buffer.irradiance_tex[engine.get_current_in_flight_frame_index()].handle.value
                } else {
                    self.temporal_denoise.temp_irradiance_tex.handle.value 
                };

                let output_tex = if i % 2 == 0 {
                    self.temporal_denoise.temp_irradiance_tex.handle.value 
                } else {
                    self.g_buffer.irradiance_tex[engine.get_current_in_flight_frame_index()].handle.value
                };

                self.temporal_denoise.filter_stage.render(buffer, AToursFilterDispatchParams {
                    g_buffer_ptr: self.g_buffer.ptr,
                    in_irradiance_tex: input_tex,
                    out_irradiance_tex: output_tex,
                    pass_index: i as _,
                }, dispatch_size);
            }
        }
 
        self.base.blit_stage.render(buffer, BlitDispatchParams {
            g_buffer_ptr: self.g_buffer.ptr,
            debug_channel: self.base.debug_channel,
            debug_depth_range: self.base.debug_depth_range,
            debug_heat_map_range: self.base.debug_heat_map_range,
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

    pub fn render_ui(&mut self, ctx: &egui::Context, camera: &Camera) { 
        egui::Window::new("Renderer")
            .default_open(true)
            .show(ctx, |ui| {
                ui.strong("Camera");
                
                let pos = camera.get_position_in_meters(); 
                ui.label(format!("Pos: ({:.02}, {:.02}, {:.02})", pos.x, pos.y, pos.z)); 
                
                let dir = camera.direction; 
                ui.label(format!("Dir: ({:.02}, {:.02}, {:.02})", dir.x, dir.y, dir.z));
                
                ui.label(format!("Frame: {}", self.g_buffer.frame_no -1));
                ui.label(format!("Steady : {}", self.g_buffer.num_steady_frames -1));

                ui.separator();
                ui.strong("Path Tracer");
                self.base.settings_ui(ui);
                self.gi.settings_ui(ui);
                self.temporal_denoise.settings_ui(ui);   
                
                ui.separator();
                ui.strong("Debug");
                self.base.debug_settings_ui(ui); 
            });
    }

    pub fn on_size_changed(
        &mut self,
        context: &Context,
        size: UVec2,
        swapchain: &Swapchain,
    ) -> OctaResult<()> {
        self.g_buffer.on_size_changed(context, &mut self.heap, size, self.base.render_into_swapchain, swapchain)?;
        self.temporal_denoise.on_size_changed(context, &mut self.heap, size)?;
        
        Ok(())
    }
}


