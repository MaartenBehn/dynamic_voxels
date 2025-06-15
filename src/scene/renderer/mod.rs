use octa_force::{camera::Camera, egui, engine::Engine, glam::UVec2, vulkan::{CommandBuffer, Context, Swapchain}, OctaResult};
use crate::{voxel_renderer::{DebugChannel, RayManagerData, VoxelRenderer}, voxel_tree64::renderer::Tree64Renderer};
use super::{Scene, SceneData};

#[derive(Debug)]
pub struct SceneRenderer {
    pub scene: Scene,
    pub voxel_renderer: VoxelRenderer,
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
        mut scene: Scene,
        camera: &Camera,
    ) -> OctaResult<SceneRenderer> {
        let mut voxel_renderer = VoxelRenderer::new::<SceneDispatchParams>(
            context, 
            swapchain, 
            camera, 
            include_bytes!("../../../slang_shaders/bin/_trace_scene.spv"))?;
        //voxel_renderer.debug_channel = DebugChannel::Depth;
        voxel_renderer.max_bounces = 0;
        voxel_renderer.temporal_denoise = false;
        voxel_renderer.denoise_counters = false;

        scene.init_buffer()?;

        Ok(SceneRenderer {
            scene,
            voxel_renderer,
            debug: true,
        })
    }

    pub fn update(&mut self, camera: &Camera, context: &Context, res: UVec2, in_flight_frame_index: usize, frame_index: usize) -> OctaResult<()> {
        self.voxel_renderer.update(camera, context, res, in_flight_frame_index, frame_index)?;

        Ok(())
    }

    pub fn render(
        &mut self,
        buffer: &CommandBuffer,
        engine: &Engine,
    ) -> OctaResult<()> {
        self.voxel_renderer.render(buffer, engine, SceneDispatchParams {
            ray_manager: self.voxel_renderer.get_ray_manager_data(),
            scene: self.scene.get_data(),
            debug: self.debug,
        })?;
        //self.debug = false;

        Ok(())
    }

    pub fn render_ui(&mut self, ctx: &egui::Context) { 
        self.voxel_renderer.render_ui(ctx);        
    }

    pub fn on_recreate_swapchain(
        &mut self,
        context: &Context,
        swapchain: &Swapchain,
    ) -> OctaResult<()> {
        self.voxel_renderer.on_recreate_swapchain(context, swapchain)?;

        Ok(())
    }
}
