use octa_force::{OctaResult, descriptor_heap::heap::{DescriptorHandleValue, ImageDescriptorHeap}, egui::{self, Ui}, glam::{IVec3, UVec2, Vec3}, vulkan::{Buffer, Context, DescriptorSet, DescriptorSetLayout, ash::vk::{self, Format}, gpu_allocator::MemoryLocation}};

use crate::{util::{buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, shader_constants::{GI_ATLAS_SIZE, PROBE_DEPTH_RES, PROBE_PADDING, PROBE_RADIANCE_RES}}, voxel::renderer::{g_buffer::ImageAndViewAndHandle, shader_stage::ShaderStage}};

pub const PROBE_PADDED_RADIANCE_RES: usize = PROBE_RADIANCE_RES + PROBE_PADDING * 2; 
pub const PROBE_PADDED_DEPTH_RES: usize = PROBE_DEPTH_RES + PROBE_PADDING * 2;
pub const GI_RADIANCE_ATLAS_RES: usize = GI_ATLAS_SIZE * PROBE_PADDED_RADIANCE_RES;
pub const GI_DEPTH_ATLAS_RES: usize = GI_ATLAS_SIZE * PROBE_PADDED_DEPTH_RES;

#[derive(Debug)]
pub struct GIRenderer {
    pub gi_probe_update_stage: ShaderStage,

    pub radiance_atlas: [ImageAndViewAndHandle; 2],
    pub depth_atlas: [ImageAndViewAndHandle; 2],
    pub active_probe_map_offset: u32,
    pub active_probe_data_offset: u32,
    pub num_active_probes: u32,
    pub only_use_probe_level: i32,

    pub active: bool,

    pub debug_probe_pos: Vec3,
    pub debug_probe_index: u32,
    pub debug_probe_depth: bool,

    pub probe_depth_bias: f32,
}

#[repr(C)]
#[derive(Debug)]
pub struct GIProbeUpdateData {
    pub radiance_atlas: [DescriptorHandleValue; 2], 
    pub depth_atlas: [DescriptorHandleValue; 2],
    pub blue_noise_tex: DescriptorHandleValue,
    pub palette: u64,
    pub start_ptr: u64,
    pub active_probe_map_offset: u32,
    pub active_probe_data_offset: u32,
    pub frame_no: u32,
    pub probe_depth_bias: f32,
}   

impl GIRenderer {
    pub fn new(
        context: &Context, 
        heap: &mut ImageDescriptorHeap,
        push_constant_size: u32,
    ) -> OctaResult<Self> {
        
        let mut create_image = |format: vk::Format|
            -> OctaResult<ImageAndViewAndHandle> {
            let flags = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED; 
            let image = context.create_image(
                flags, 
                MemoryLocation::GpuOnly, 
                format, 
                UVec2::splat(GI_RADIANCE_ATLAS_RES as _))?;

            let view = image.create_image_view(false)?;

            let handle = heap.create_image_handle(&view, flags)?;
            
            Ok(ImageAndViewAndHandle {
                image,
                view,
                handle,
            })
        };

        let radiance_atlas= [
            create_image(Format::R8G8B8A8_UNORM)?,
            create_image(Format::R8G8B8A8_UNORM)?
        ];

        let depth_atlas= [
            create_image(Format::R32_SFLOAT)?,
            create_image(Format::R32_SFLOAT)?
        ]; 

        let sets = &[&heap.layout];
        let gi_probe_update_stage = ShaderStage::new(
            context, 
            include_bytes!(concat!(env!("OUT_DIR"),"/_gi_probe_update_main.spv")), 
            sets, push_constant_size)?;

                       
        Ok(Self {
            radiance_atlas,
            depth_atlas,
            gi_probe_update_stage,
            active_probe_map_offset: 0,
            active_probe_data_offset: 0,
            num_active_probes: 0,
            only_use_probe_level: -1,
            active: false,
            debug_probe_pos: Vec3::ZERO,
            debug_probe_index: 0,
            debug_probe_depth: false,
            probe_depth_bias: 0.03,
        })
    }

    pub fn settings_ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.active, "Use Probes");
        ui.add(egui::Slider::new(&mut self.probe_depth_bias, 0.001..=0.5)
            .text("Probe Depth Bias")
        );
        ui.label(format!("Active Probes: {}", self.num_active_probes));
        ui.label(format!("Debug Probe: {} {}", self.debug_probe_index, self.debug_probe_pos));
        ui.checkbox(&mut self.debug_probe_depth, "Debug Probe Depth");
        ui.add(egui::Slider::new(&mut self.only_use_probe_level, -1..=10)
            .text("Only Use Probe Level")
        );
    }

}
