use octa_force::{OctaResult, egui::{self, Align, Frame, Layout, Ui}, glam::UVec2, vulkan::{Context, DescriptorSet, DescriptorSetLayout, Swapchain, ash::vk::{self, Format}, descriptor_heap::{DescriptorHandleValue, ImageDescriptorHeap}, gpu_allocator::MemoryLocation}};

use crate::voxel::renderer::{g_buffer::ImageAndViewAndHandle, shader_stage::ShaderStage};


#[derive(Debug)]
pub struct TemporalDenoiseRenderer {
    pub temp_irradiance_tex: ImageAndViewAndHandle,

    pub denoise_stage: ShaderStage,
    pub filter_stage: ShaderStage,

    pub temporal_denoise: bool,
    pub denoise_counters: bool,
    pub static_accum_number: u32,
    pub filter_passes: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct TemporalDenoiseDispatchParams {
    pub g_buffer_ptr: u64,
    pub static_accum_number: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct AToursFilterDispatchParams {
    pub g_buffer_ptr: u64,
    pub in_irradiance_tex: DescriptorHandleValue,
    pub out_irradiance_tex: DescriptorHandleValue,
    pub pass_index: u32,
}

impl TemporalDenoiseRenderer {
    pub fn new(
        context: &Context, 
        swapchain: &Swapchain,
        heap: &mut ImageDescriptorHeap,
        push_constant_size: u32,
    ) -> OctaResult<Self> {
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
        let denoise_stage = ShaderStage::new(
            context, 
            include_bytes!(concat!(env!("OUT_DIR"),"/_temporal_denoise_main.spv")), 
            sets, push_constant_size)?;

        let filter_stage = ShaderStage::new(
            context, 
            include_bytes!(concat!(env!("OUT_DIR"),"/_a_tours_filter_main.spv")), 
            sets, push_constant_size)?;

        Ok(Self {
            temp_irradiance_tex,
            denoise_stage,
            filter_stage,
            temporal_denoise: true,
            denoise_counters: true,
            filter_passes: 2,
            static_accum_number: 10,
        })
    }

    pub fn settings_ui(&mut self, ui: &mut Ui) {
        div(ui, |ui| {
            ui.checkbox(&mut self.temporal_denoise, "Temporal Denoise");
            ui.checkbox(&mut self.denoise_counters, "Jitter");
        });

        ui.add(egui::Slider::new(&mut self.static_accum_number, 10..=255)
            .text("Static Accum Number")
        );

        ui.separator();
        ui.strong("A-tours filter");

        ui.add(egui::Slider::new(&mut self.filter_passes, 0..=10)
            .text("Passes")
        );
        if self.filter_passes % 2 == 1 {
            self.filter_passes += 1;
        }
    }

    pub fn on_size_changed(
        &mut self, 
        context: &Context, 
        heap: &mut ImageDescriptorHeap,
        size: UVec2, 
    ) -> OctaResult<()> {
        let temp_irradiance_image = context.create_image(
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC, 
            MemoryLocation::GpuOnly, 
            Format::R8G8B8A8_UNORM, 
            size)?;

        let temp_irradiance_view = temp_irradiance_image.create_image_view(false)?;

        let temp_irradiance_handle = heap.create_image_handle(&temp_irradiance_view, vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC)?;

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
