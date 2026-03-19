use octa_force::{OctaResult, egui::Ui, glam::{IVec3, UVec2}, vulkan::{Buffer, Context, DescriptorSet, DescriptorSetLayout, ash::vk::{self, Format}, descriptor_heap::{DescriptorHandleValue, ImageDescriptorHeap}, gpu_allocator::MemoryLocation}};

use crate::{util::buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, voxel::renderer::{g_buffer::ImageAndViewAndHandle, shader_stage::ShaderStage}};

pub const GI_ATLAS_SIZE: usize = 64;
const PROBE_RADIANCE_RES: usize = 8;
const PROBE_DEPTH_RES: usize = 8;

#[derive(Debug)]
pub struct GIRenderer {
    pub gi_probe_update_stage: ShaderStage,

    pub radiance_atlas: ImageAndViewAndHandle,
    pub depth_atlas: ImageAndViewAndHandle,
    pub probes_offset: u32,
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
    pub probes_offset: u32,
}

impl GIRenderer {
    pub fn new(
        context: &Context, 
        heap: &mut ImageDescriptorHeap,
        push_constant_size: u32,
    ) -> OctaResult<Self> {
        
        let atlas_side_size = GI_ATLAS_SIZE;

        let flags = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED; 
        let radiance_atlas_image= context.create_image(
            flags, 
            MemoryLocation::GpuOnly, 
            Format::R8G8B8_UNORM, 
            UVec2::splat((atlas_side_size * PROBE_RADIANCE_RES + 2) as _))?;
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
            UVec2::splat((atlas_side_size * PROBE_DEPTH_RES + 2) as _))?;
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
            include_bytes!("../../../shaders/bin/_gi_probe_update_main.spv"), 
            sets, push_constant_size)?;

                       
        Ok(Self {
            radiance_atlas,
            depth_atlas,
            gi_probe_update_stage,
            probes_offset: 0,
            num_active_probes: 0,
            active: true,
        })
    }

    pub fn settings_ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.active, "Use Probes");
        ui.label(format!("Active Probes: {}", self.num_active_probes));
    }

}
