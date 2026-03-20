use octa_force::{OctaResult, egui::Ui, glam::{IVec3, UVec2}, vulkan::{Buffer, Context, DescriptorSet, DescriptorSetLayout, ash::vk::{self, Format}, descriptor_heap::{DescriptorHandleValue, ImageDescriptorHeap}, gpu_allocator::MemoryLocation}};

use crate::{util::{buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, shader_constants::{GI_ATLAS_SIZE, PROBE_DEPTH_RES, PROBE_PADDING, PROBE_RADIANCE_RES}}, voxel::renderer::{g_buffer::ImageAndViewAndHandle, shader_stage::ShaderStage}};

pub const PROBE_PADDED_RADIANCE_RES: usize = PROBE_RADIANCE_RES + PROBE_PADDING * 2; 
pub const PROBE_PADDED_DEPTH_RES: usize = PROBE_DEPTH_RES + PROBE_PADDING * 2;
pub const GI_RADIANCE_ATLAS_RES: usize = GI_ATLAS_SIZE * PROBE_PADDED_RADIANCE_RES;
pub const GI_DEPTH_ATLAS_RES: usize = GI_ATLAS_SIZE * PROBE_PADDED_DEPTH_RES;

#[derive(Debug)]
pub struct GIRenderer {
    pub gi_probe_update_stage: ShaderStage,

    pub radiance_atlas: ImageAndViewAndHandle,
    pub depth_atlas: ImageAndViewAndHandle,
    pub active_probe_map_offset: u32,
    pub active_probe_data_offset: u32,
    pub num_active_probes: u32,

    pub active: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct GIProbeUpdateData {
    pub radiance_atlas: DescriptorHandleValue, 
    pub depth_atlas: DescriptorHandleValue,
    pub palette: u64,
    pub start_ptr: u64,
    pub active_probe_data_offset: u32,
}

impl GIRenderer {
    pub fn new(
        context: &Context, 
        heap: &mut ImageDescriptorHeap,
        push_constant_size: u32,
    ) -> OctaResult<Self> {
        
        dbg!(GI_RADIANCE_ATLAS_RES);
        let flags = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED; 
        let radiance_atlas_image= context.create_image(
            flags, 
            MemoryLocation::GpuOnly, 
            Format::R8G8B8A8_UNORM, 
            UVec2::splat(GI_RADIANCE_ATLAS_RES as _))?;
        let radiance_atlas_view = radiance_atlas_image.create_image_view(false)?;
        let radiance_atlas_handle = heap.create_image_handle(&radiance_atlas_view, flags)?;
        let radiance_atlas = ImageAndViewAndHandle { 
            image: radiance_atlas_image, 
            view: radiance_atlas_view, 
            handle: radiance_atlas_handle 
        };

        let depth_atlas_image= context.create_image(
            flags, 
            MemoryLocation::GpuOnly, 
            Format::R32_SFLOAT, 
            UVec2::splat(GI_DEPTH_ATLAS_RES as _))?;
        let depth_atlas_view = depth_atlas_image.create_image_view(false)?;
        let depth_atlas_handle = heap.create_image_handle(&depth_atlas_view, flags)?;
        let depth_atlas = ImageAndViewAndHandle { 
            image: depth_atlas_image, 
            view: depth_atlas_view, 
            handle: depth_atlas_handle 
        };

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
            active: true,
        })
    }

    pub fn settings_ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.active, "Use Probes");
        ui.label(format!("Active Probes: {}", self.num_active_probes));
    }

}
