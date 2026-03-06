pub mod static_dag64_buffer;

use octa_force::anyhow::Result;
use octa_force::camera::Camera;
use octa_force::engine::Engine;
use octa_force::glam::UVec2;
use octa_force::vulkan::{
    CommandBuffer, Context, Swapchain
};
use octa_force::{egui, OctaResult};
use static_dag64_buffer::{StaticDAG64Buffer, StaticDAG64Data};

use crate::voxel::palette::shared::SharedPalette;
use crate::voxel::renderer::{RayManagerData, VoxelRenderer};

use super::StaticVoxelDAG64;

const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 8;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 8;

#[allow(dead_code)]
#[derive(Debug)]
pub struct StaticDAG64Renderer {
    voxel_tree64_buffer: StaticDAG64Buffer,
    voxel_renderer: VoxelRenderer,
}

#[repr(C)]
#[derive(Debug)]
pub struct TraceStaticDAG64DispatchParams {
    ray_manager: RayManagerData,
    tree64: StaticDAG64Data,
}

impl StaticDAG64Renderer {
    pub fn new(
        context: &Context,
        swapchain: &Swapchain,
        tree: StaticVoxelDAG64,
        camera: &Camera,
        palette: SharedPalette,
    ) -> Result<StaticDAG64Renderer> {
 
        let voxel_renderer = VoxelRenderer::new::<TraceStaticDAG64DispatchParams>(
            context,
            swapchain, 
            camera, 
            palette,
            include_bytes!("../../../../shaders/bin/_trace_tree64_main.spv"),
            true,
        )?;

        let voxel_tree64_buffer = tree.into_buffer(context)?;
        
        Ok(StaticDAG64Renderer {
            voxel_tree64_buffer,
            voxel_renderer,
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
        self.voxel_renderer.render(UVec2::ZERO, buffer, engine, TraceStaticDAG64DispatchParams {
            ray_manager: self.voxel_renderer.get_ray_manager_data(),
            tree64: self.voxel_tree64_buffer.get_data(),
        })?;

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
        self.voxel_renderer.on_size_changed(context, swapchain.size, swapchain)?;

        Ok(())
    }
}
