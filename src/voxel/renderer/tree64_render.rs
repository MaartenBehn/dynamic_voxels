use std::time::Instant;

use octa_force::{OctaResult, camera::Camera, egui, engine::Engine, glam::{Mat4, UVec2, Vec3A, vec3a}, log::info, vulkan::{Buffer, CommandBuffer, Context, Swapchain, ash::vk}};

use crate::{VOXELS_PER_METER, util::{aabb::AABB, default_types::{LODType, Volume}}, volume::VolumeBounds, voxel::{dag64::{DAG64Entry, VoxelDAG64, lod_heuristic::{LODHeuristicT, PowHeuristicSphere}, parallel::ParallelVoxelDAG64}, palette::{Palette, palette::MATERIAL_ID_BASE, shared::SharedPalette}, renderer::{RayManagerData, VoxelRenderer}}};

#[derive(Debug)]
pub struct Tree64Renderer {
    csg: Volume,
    dag: ParallelVoxelDAG64,
    node_buffer: Buffer,
    data_buffer: Buffer,
    start_index: u32,
    voxel_renderer: VoxelRenderer,
    mat: Mat4,
    inv_mat: Mat4,
}

#[repr(C)]
pub struct Tree64RendererData {
    mat: Mat4,
    inv_mat: Mat4,
    ray_manager: RayManagerData,
    tree: Tree64Data,
}

#[repr(C)]
pub struct Tree64Data {
    pub nodes_ptr: u64,
    pub data_ptr: u64,
    pub start_index: u32,
}

impl Tree64Renderer {
    pub fn new(engine: &Engine, camera: &Camera) -> OctaResult<Self> {

        let factor = 100.0;
        let mut csg = Volume::new_sphere_float(Vec3A::ZERO, 
            100.0 * factor, MATERIAL_ID_BASE);
        csg.cut_with_sphere(vec3a(70.0 * factor, 0.0, 0.0), 70.0 * factor, MATERIAL_ID_BASE);

        let mut lod = PowHeuristicSphere::default();
        lod.render_dist = 50.0;
        lod.set_center((camera.get_position_in_meters() * VOXELS_PER_METER as f32).as_ivec3());

        let now = Instant::now();

        let mut dag = ParallelVoxelDAG64::new(
            200000000, 
            10000, 
        );
        dag.print_memory_info();
        let key = dag.add_aabb_query_volume(&csg, &lod);

        let elapsed = now.elapsed();
        info!("Tree Build took {:.2?}", elapsed);
    
        dag.print_memory_info();

        let entry = dag.get_entry(key);

        let mat = entry.calc_mat(Mat4::IDENTITY);
        let inv_mat = mat.inverse();

        let node_buffer = engine.context.create_gpu_only_buffer_from_data(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR, 
            &dag.nodes.data())?;

        let data_buffer = engine.context.create_gpu_only_buffer_from_data(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR, 
            &dag.data.data())?;

        let palette = SharedPalette::new();

        let mut voxel_renderer = VoxelRenderer::new::<Tree64RendererData>(
            &engine.context,
            &engine.swapchain, 
            camera,
            palette,
            include_bytes!("../../../shaders/bin/_trace_tree64_main.spv"),
            true,
        )?;
        voxel_renderer.max_bounces = 0;
        voxel_renderer.temporal_denoise = false;
        voxel_renderer.denoise_counters = false;

        Ok(Self {
            csg,
            dag,
            node_buffer,
            data_buffer,
            start_index: entry.root_index,
            voxel_renderer,
            mat, 
            inv_mat,
        })
    }

    pub fn update(&mut self, engine: &Engine, camera: &Camera) 
        -> OctaResult<()> {

        self.voxel_renderer.update(
            camera, 
            &engine.context, 
            engine.get_resolution(),
            engine.get_current_in_flight_frame_index(),
            engine.get_current_frame_index()
        )?;

        if engine.controls.f2 {
            
            let mut lod = LODType::default();
            lod.set_center((camera.get_position_in_meters() * VOXELS_PER_METER as f32).as_ivec3());

            let now = Instant::now();

            let key = self.dag.add_aabb_query_volume(&self.csg, &lod);

            let elapsed = now.elapsed();
            info!("Tree Build took {:.2?}", elapsed);
            self.dag.print_memory_info();

            let entry = self.dag.get_entry(key);
            
            self.start_index = entry.root_index;
            self.mat = entry.calc_mat(Mat4::IDENTITY);
            self.inv_mat = self.mat.inverse();

            self.node_buffer = engine.context.create_gpu_only_buffer_from_data(
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR, 
                &self.dag.nodes.data())?;

            self.data_buffer = engine.context.create_gpu_only_buffer_from_data(
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR, 
                &self.dag.data.data())?;
        }

        Ok(())
    }

    pub fn render(
        &mut self,
        buffer: &CommandBuffer,
        engine: &Engine,
        camera: &Camera,
    ) -> OctaResult<()> {
        self.voxel_renderer.render(UVec2::ZERO, buffer, engine, Tree64RendererData {
            mat: self.mat,
            inv_mat: self.inv_mat,
            ray_manager: self.voxel_renderer.get_ray_manager_data(),
            tree: Tree64Data {
                nodes_ptr: self.node_buffer.get_device_address(), 
                data_ptr: self.data_buffer.get_device_address(),
                start_index: self.start_index,
            }
        })?;
       
        Ok(())
    }

    pub fn render_ui(&mut self, ctx: &egui::Context) { 
        self.voxel_renderer.render_ui(ctx);        
    }

    pub fn on_size_changed(
        &mut self,
        engine: &Engine,
    ) -> OctaResult<()> {
        self.voxel_renderer.on_size_changed(&engine.context, engine.get_resolution(), &engine.swapchain)
    }
}


