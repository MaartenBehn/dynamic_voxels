use std::{mem, time::Instant};

use octa_force::{OctaResult, camera::Camera, egui, engine::Engine, glam::{UVec2, Vec3}, log::{debug, info}, vulkan::{Buffer, CommandBuffer, CommandPool, Context, Fence, Swapchain, ash::vk::{self, AttachmentLoadOp}, gpu_allocator::MemoryLocation}};
use crate::{VOXELS_PER_METER, scene::{staging_copies::SceneStaging, worker::SceneWorkerRef}, util::math::to_mb, voxel::{palette::shared::SharedPalette, renderer::{DebugChannel, RayManagerData, VoxelRenderer}}};

use super::{worker::{SceneWorker}};

pub const NUM_SCENE_GPU_BUFFERS: usize = 2;

#[derive(Debug)]
pub struct SceneRenderer {
    pub worker_ref: SceneWorkerRef,
    pub last_send_camera_position: Vec3,
    
    pub gpu_buffers: [Buffer; NUM_SCENE_GPU_BUFFERS],
    pub gpu_buffers_addresses: [u64; NUM_SCENE_GPU_BUFFERS],
    pub rendered_gpu_buffer_index: usize,
    pub copy_gpu_buffer_index: usize,
   
    current_staging: Option<SceneStaging>,
    staging_state: StagingState,
    staging_fence: Fence,
    staging_command_pool: CommandPool,
    staging_command_buffer: CommandBuffer,
    start_staging_copy_time: Instant,

    pub voxel_renderer: VoxelRenderer,
    pub render_data: SceneData,
    pub debug: bool,

    pub update_camera: bool,
}

#[derive(Debug)]
enum StagingState {
    Inactive,
    CopyToFirst,
    CopyToSecond,
}

#[repr(C)]
pub struct SceneDispatchParams {
    pub ray_manager: RayManagerData,
    pub scene: SceneData,
    pub debug: bool,
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
        engine: &Engine,
        camera: &Camera,
        palette: SharedPalette,
        allways_fullscreen: bool,
    ) -> OctaResult<SceneRenderer> {

        let gpu_buffer_size = 2_usize.pow(30);
        info!("Scene Buffer size: {:.04} MB", to_mb(gpu_buffer_size));
  
        let flags = vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR | vk::BufferUsageFlags::TRANSFER_DST;

        let gpu_buffers = [
            engine.context.create_buffer(
                flags,
                MemoryLocation::GpuOnly,
                gpu_buffer_size as _)?,
            engine.context.create_buffer(
                flags,
                MemoryLocation::GpuOnly,
                gpu_buffer_size as _)?,
        ];
        let gpu_buffers_addresses = [
            gpu_buffers[0].get_device_address(),
            gpu_buffers[1].get_device_address(),
        ];

        let mut voxel_renderer = VoxelRenderer::new::<SceneDispatchParams>(
            &engine.context,
            &engine.swapchain, 
            camera,
            palette,
            include_bytes!("../../shaders/bin/_trace_scene_main.spv"),
            allways_fullscreen,
        )?;

        voxel_renderer.max_bounces = 0;
        voxel_renderer.temporal_denoise = false;
        voxel_renderer.denoise_counters = false;

        let worker = SceneWorker::new(gpu_buffer_size, &engine.context)?;

        let render_data = SceneData {
            start_ptr: gpu_buffers_addresses[0],
            bvh_offset: worker.bvh_allocation.start() as u32,
            bvh_len: worker.bvh_len as u32,
        };

        let worker_ref = worker.run();
        worker_ref.send.camera_position(camera.get_position_in_meters());

        let staging_fence = engine.context.create_fence(None)?;
        let staging_command_pool = engine.context.create_command_pool(
            engine.context.physical_device.graphics_queue_family,
            Some(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
        )?;

        let staging_command_buffer = staging_command_pool
                            .allocate_command_buffer(vk::CommandBufferLevel::PRIMARY)?;

        Ok(SceneRenderer {
            worker_ref,
            last_send_camera_position: camera.get_position_in_meters(), 

            gpu_buffers,
            gpu_buffers_addresses,
            rendered_gpu_buffer_index: 0,
            copy_gpu_buffer_index: 1,

            current_staging: None,
            staging_state: StagingState::Inactive,
            staging_fence,
            staging_command_pool,
            staging_command_buffer,
            start_staging_copy_time: Instant::now(),

            voxel_renderer,
            render_data,
            debug: true,

            update_camera: false,
        })
    }

    fn copy_staging(&self, staging: &SceneStaging, engine: &Engine) -> OctaResult<()> {

        self.staging_command_buffer.reset()?;
        self.staging_command_buffer.begin(Some(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT))?;

        self.staging_command_buffer.copy_buffer_regions(
            &staging.buffer, 
            &self.gpu_buffers[self.copy_gpu_buffer_index], 
            &staging.regions);

        self.staging_command_buffer.end()?;

        engine.context.graphics_queue
            .submit(&self.staging_command_buffer, 
                None, 
                None, 
                &self.staging_fence)?;

        Ok(())
    }

    pub fn update(&mut self, engine: &Engine, camera: &Camera, size: UVec2) 
        -> OctaResult<()> {

        self.voxel_renderer.update(
            camera, 
            &engine.context, 
            size,
            engine.get_current_in_flight_frame_index(),
            engine.get_current_frame_index()
        )?;

        match self.staging_state {
            StagingState::Inactive => {
                if let Ok(staging) = self.worker_ref.render_r.try_recv() {

                    self.start_staging_copy_time = Instant::now();
                    self.copy_staging(&staging, engine)?;

                    self.current_staging = Some(staging);
                    self.staging_state = StagingState::CopyToFirst;
                } 
            },
            StagingState::CopyToFirst => {
                if self.staging_fence.is_done()? {
                    self.staging_fence.reset()?;

                    let took = self.start_staging_copy_time.elapsed();
                    debug!("First buffer copy took: {took:?}");

                    self.rendered_gpu_buffer_index = (self.rendered_gpu_buffer_index + 1) % NUM_SCENE_GPU_BUFFERS;
                    self.copy_gpu_buffer_index = (self.copy_gpu_buffer_index + 1) % NUM_SCENE_GPU_BUFFERS;

                    let staging = self.current_staging.as_ref().unwrap();
                    self.render_data.start_ptr = self.gpu_buffers_addresses[self.rendered_gpu_buffer_index];
                    self.render_data.bvh_offset = staging.bvh_offset;
                    self.render_data.bvh_len = staging.bvh_len;
                    
                    self.start_staging_copy_time = Instant::now();
                    self.copy_staging(staging, engine)?;
                    
                    self.staging_state = StagingState::CopyToSecond;
                }
            },
            StagingState::CopyToSecond => {
                if self.staging_fence.is_done()? {
                    self.staging_fence.reset()?;

                    let took = self.start_staging_copy_time.elapsed();
                    debug!("Second buffer copy took: {took:?}");

                    let staging = self.current_staging.take().unwrap();
                    self.worker_ref.send.free_staging_buffer(staging.buffer);

                    self.staging_state = StagingState::Inactive; 
                }
            },
        }

        if self.update_camera {
            let new_cam_pos = camera.get_position_in_meters(); 
            if (self.last_send_camera_position.distance(new_cam_pos) * VOXELS_PER_METER as f32) > 100.0 {
                self.worker_ref.send.camera_position(new_cam_pos);
                self.last_send_camera_position = new_cam_pos;
            }
        }
        
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
            scene: self.render_data,
            debug: self.debug,
        })?;
        //self.debug = false;
       
        Ok(())
    }

    pub fn render_ui(&mut self, ctx: &egui::Context, camera: &Camera) { 
        self.voxel_renderer.render_ui(ctx, camera);        
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
