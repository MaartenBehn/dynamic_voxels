use octa_force::{camera::Camera, egui, engine::Engine, glam::UVec2, vulkan::{CommandBuffer, Context, Swapchain}, OctaResult};
use crate::voxel::{palette::shared::SharedPalette, renderer::{RayManagerData, VoxelRenderer}};

use super::{worker::{SceneWorker, SceneWorkerRenderData}, Scene, SceneData};

#[derive(Debug)]
pub struct SceneRenderer {
    pub voxel_renderer: VoxelRenderer,
    pub scene_render_data: SceneWorkerRenderData,
    pub debug: bool,
}

#[repr(C)]
pub struct SceneDispatchParams {
    ray_manager: RayManagerData,
    scene: SceneData,
    debug: bool,
}

impl SceneRenderer {
    pub fn new(
        context: &Context,
        swapchain: &Swapchain,
        camera: &Camera,
        scene_render_data: SceneWorkerRenderData,
        palette: SharedPalette,
        allways_fullscreen: bool,
    ) -> OctaResult<SceneRenderer> {
        let mut voxel_renderer = VoxelRenderer::new::<SceneDispatchParams>(
            context,
            swapchain, 
            camera,
            palette,
            include_bytes!("../../../shaders/slang/bin/_trace_scene.spv"),
            allways_fullscreen,
        )?;

        voxel_renderer.max_bounces = 0;
        voxel_renderer.temporal_denoise = false;
        voxel_renderer.denoise_counters = false;

        Ok(SceneRenderer {
            voxel_renderer,
            scene_render_data,
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
