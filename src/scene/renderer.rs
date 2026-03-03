use octa_force::{OctaResult, camera::Camera, egui, engine::Engine, glam::UVec2, log::info, vulkan::{Buffer, CommandBuffer, Context, Swapchain, ash::vk::{self, AttachmentLoadOp}, gpu_allocator::MemoryLocation}};
use crate::{scene::worker::SceneWorkerRef, util::math::to_mb, voxel::{palette::shared::SharedPalette, renderer::{RayManagerData, VoxelRenderer}}};

use super::{worker::{SceneWorker}};

pub const NUM_SCENE_GPU_BUFFERS: usize = 2;

#[derive(Debug)]
pub struct SceneRenderer {
    pub worker_ref: SceneWorkerRef,
    
    pub gpu_buffers: [Buffer; NUM_SCENE_GPU_BUFFERS],
    pub gpu_buffers_addresses: [u64; NUM_SCENE_GPU_BUFFERS],
    pub rendered_gpu_buffer_index: usize,

    pub voxel_renderer: VoxelRenderer,
    pub debug: bool,
}

#[repr(C)]
pub struct SceneDispatchParams {
    ray_manager: RayManagerData,
    scene: SceneData,
    debug: bool,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SceneData {
    start_ptr: u64,
    bvh_offset: u32,
    bvh_len: u32,
}

impl SceneRenderer {
    pub fn new(
        context: &Context,
        swapchain: &Swapchain,
        camera: &Camera,
        palette: SharedPalette,
        allways_fullscreen: bool,
    ) -> OctaResult<SceneRenderer> {

        let gpu_buffer_size = 2_usize.pow(20);
        info!("Scene Buffer size: {:.04} MB", to_mb(gpu_buffer_size));

        let worker = SceneWorker::new(gpu_buffer_size, context)?;
        let worker_ref = worker.run();
        

        let gpu_buffers = [
            context.create_buffer(
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
                MemoryLocation::GpuOnly,
                gpu_buffer_size as _)?,
            context.create_buffer(
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
                MemoryLocation::GpuOnly,
                gpu_buffer_size as _)?,
        ];
        let gpu_buffers_addresses = [
            gpu_buffers[0].get_device_address(),
            gpu_buffers[1].get_device_address(),
        ];

        let mut voxel_renderer = VoxelRenderer::new::<SceneDispatchParams>(
            context,
            swapchain, 
            camera,
            palette,
            include_bytes!("../../../shaders/bin/_trace_scene_main.spv"),
            allways_fullscreen,
        )?;

        voxel_renderer.max_bounces = 0;
        voxel_renderer.temporal_denoise = false;
        voxel_renderer.denoise_counters = false;

        Ok(SceneRenderer {
            worker_ref,

            gpu_buffers,
            gpu_buffers_addresses,
            rendered_gpu_buffer_index: 0,

            voxel_renderer,
            debug: true,
        })
    }

    pub fn update(&mut self, camera: &Camera, context: &Context, size: UVec2, in_flight_frame_index: usize, frame_index: usize) -> OctaResult<()> {
        self.voxel_renderer.update(camera, context, size, in_flight_frame_index, frame_index)?;
        Ok(())
    }

    pub fn render(
        &mut self,
        offset: UVec2,
        buffer: &CommandBuffer,
        engine: &Engine,
        camera: &Camera,
    ) -> OctaResult<()> {
        self.voxel_renderer.render(offset, buffer, engine, SceneDispatchParams {
            ray_manager: self.voxel_renderer.get_ray_manager_data(),
            scene: self.scene_render_data.get_data(),
            debug: self.debug,
        })?;
        //self.debug = false;
       
        Ok(())
    }

    pub fn render_ui(&mut self, ctx: &egui::Context) { 
        self.voxel_renderer.render_ui(ctx);        
    }

    pub fn on_size_changed(
        &mut self,
        size: UVec2,
        context: &Context,
        swapchain: &Swapchain,
    ) -> OctaResult<()> {
        self.voxel_renderer.on_size_changed(context, size, swapchain)?;

        Ok(())
    }
}
