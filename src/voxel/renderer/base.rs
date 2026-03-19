use egui_double_slider::DoubleSlider;
use octa_force::{OctaResult, egui::{self, Align, Frame, Layout, Ui}, glam::{UVec2, Vec2, vec2}, image::{GenericImageView, ImageReader}, vulkan::{Context, DescriptorSet, DescriptorSetLayout, ash::vk::{self, Format}, descriptor_heap::{DescriptorHandleValue, ImageDescriptorHeap}}};
use spirv_struct_layout::SpirvLayout;

use crate::voxel::renderer::{g_buffer::ImageAndViewAndHandle, shader_stage::ShaderStage};

#[derive(Debug)]
pub struct BaseRenderer {
    pub blue_noise_tex: ImageAndViewAndHandle,
   
    pub trace_scene_stage: ShaderStage,
    pub blit_stage: ShaderStage,

    pub max_bounces: u32,
    pub start_ptr: u64,
    pub bvh_offset: u32,
    pub bvh_len: u32,

    pub debug_channel: DebugChannel,
    pub debug_depth_range: Vec2,
    pub debug_heat_map_range: Vec2,

    pub render_into_swapchain: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct SceneDispatchDispatchParams {
    pub g_buffer_ptr: u64,
    pub palette_ptr: u64,
    pub max_bounces: u32,
    pub blue_noise_tex: DescriptorHandleValue,
    pub start_ptr: u64,
    pub bvh_offset: u32,
    pub bvh_len: u32,
    pub debug: bool,
}

#[repr(C)]
#[derive(SpirvLayout)]
#[derive(Debug)]
pub struct BlitDispatchParams {
    pub g_buffer_ptr: u64,
    pub debug_depth_range: Vec2,
    pub debug_heat_map_range: Vec2,
    pub debug_channel: DebugChannel,
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

impl BaseRenderer {
    pub fn new(
        context: &Context, 
        heap: &mut ImageDescriptorHeap,
        push_constant_size: u32,
        render_into_swapchain: bool,
    ) -> OctaResult<Self> {

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
        let trace_scene_stage = ShaderStage::new(
            context, 
            include_bytes!("../../../shaders/bin/_trace_scene_main.spv"), 
            sets, push_constant_size)?;

        let blit_stage = ShaderStage::new(
            context, 
            include_bytes!("../../../shaders/bin/_blit_main.spv"), 
            sets, push_constant_size)?;

        Ok(Self {
            blue_noise_tex,
            trace_scene_stage,
            blit_stage,
            debug_channel: DebugChannel::None,
            render_into_swapchain,
            start_ptr: 0,
            bvh_offset: 0,
            bvh_len: 0,

            max_bounces: 0,
            debug_depth_range: vec2(0.5, 1.1),
            debug_heat_map_range: vec2(0.0, 100.0),
        })
    }

    pub fn settings_ui(&mut self, ui: &mut Ui) {

        ui.add(egui::Slider::new(&mut self.max_bounces, 0..=10)
            .text("Max Bounces")
        );
    }

    pub fn debug_settings_ui(&mut self, ui: &mut Ui) {

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
    }
}


