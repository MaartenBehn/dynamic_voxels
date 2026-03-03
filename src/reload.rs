#![forbid(unused_must_use)]

extern crate self as dynamic_voxels;

pub mod csg;
pub mod model;
pub mod multi_data_buffer;
pub mod scene;
pub mod util;
pub mod volume;
pub mod voxel;
pub mod bvh;
pub mod mesh;
pub mod marching_cubes;

use csg::csg_tree::tree::CSGTree;
use csg::union::tree::{Union, UnionNode};
use model::composer::ModelComposer;
use octa_force::engine::Engine;
use parking_lot::Mutex;
use scene::dag64::SceneAddDAGObject;
use scene::renderer::SceneRenderer;
use scene::worker::SceneWorker;
use slotmap::Key;
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, vec3a, DVec3, EulerRot, IVec2, IVec3, Mat4, Quat, UVec2, UVec3, Vec3, Vec3A};
use octa_force::gui::Gui;
use octa_force::log::{debug, error, info, trace, LevelFilter, Log};
use octa_force::logger::setup_logger;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use octa_force::vulkan::{Context, Fence, ImageBarrier};
use octa_force::{egui, log, OctaResult};
use util::profiler::ShaderProfiler;
use util::state_saver::StateSaver;
use volume::{VolumeBounds};
use voxel::dag64::VoxelDAG64;
use voxel::grid::VoxelGrid;
use voxel::palette::shared::SharedPalette;
use voxel::static_dag64::renderer::StaticDAG64Renderer;
use voxel::static_dag64::StaticVoxelDAG64;
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{default, env};

#[cfg(any(feature="graph"))]
use crate::mesh::scene::MeshScene;
#[cfg(any(feature="scene"))]
use crate::scene::dag_store::{LODType, SceneDAGKey};
#[cfg(any(feature="scene"))]
use crate::voxel::dag64::parallel::ParallelVoxelDAG64;

pub const USE_PROFILE: bool = false;
pub const NUM_FRAMES_IN_FLIGHT: usize = 2;

pub const VOXELS_PER_METER: usize = 10;
pub const METERS_PER_SHADER_UNIT: usize = 1000;
pub const VOXELS_PER_SHADER_UNIT: usize = VOXELS_PER_METER * METERS_PER_SHADER_UNIT;

#[unsafe(no_mangle)]
pub fn init_hot_reload(logger: &'static dyn Log, level: LevelFilter) -> OctaResult<()> {
    setup_logger(logger, level)?;
 
    Ok(())
}

#[derive(Debug)]
pub struct LogicState {
    pub camera: Camera,
    pub start_time: Instant,
}

#[unsafe(no_mangle)]
pub fn new_logic_state() -> OctaResult<LogicState> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    log::info!("Creating Camera");
    let mut camera = Camera::default();
 
    #[cfg(feature="scene")]
    {
        camera.set_meter_per_unit(METERS_PER_SHADER_UNIT as f32);
        camera.set_position_in_meters(Vec3::new(30.0, 30.0, 20.0)); 
        camera.direction = Vec3::new(-0.6110025, -0.7362994, -0.29075617).normalize();
        
        camera.speed = 10.0;
        camera.z_near = 0.001;
    }

    #[cfg(feature="graph")]
    {
        camera.set_meter_per_unit(METERS_PER_SHADER_UNIT as f32);
        camera.set_position_in_meters(Vec3::new(0.0, -10.0, 0.0)); 
        camera.direction = Vec3::new(0.0, 1.0, 0.0).normalize();
        
        camera.speed = 50.0;
        camera.z_near = 0.001;
    }
 
    camera.z_far = 100.0;
    camera.up = vec3(0.0, 0.0, 1.0);

    Ok(LogicState {
        camera,
        start_time: Instant::now(),
    })
}


#[derive(Debug)]
pub struct RenderState {
    #[cfg(any(feature="scene", feature="graph"))]
    pub scene: SceneRenderer,
    
    #[cfg(any(feature="scene"))]
    pub dag: ParallelVoxelDAG64<LODType>,
    
    #[cfg(any(feature="scene"))]
    pub csg: CSGTree<u8, IVec3, i32, 3>, 

    #[cfg(any(feature="scene"))]
    pub dag_key: SceneDAGKey,
    
    #[cfg(any(feature="scene"))]
    pub object_key: SceneObjectKey,
    
    #[cfg(any(feature="graph"))]
    pub composer: ModelComposer, 
    
    #[cfg(any(feature="graph"))]
    pub mesh_scene: MeshScene,
}

#[unsafe(no_mangle)]
pub fn new_render_state(logic_state: &mut LogicState, engine: &mut Engine) -> OctaResult<RenderState> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();
       
    #[cfg(feature="scene")]
    {
        use octa_force::glam::ivec3;

        use crate::{util::vector::Ve, voxel::{dag64::lod_heuristic::{LODHeuristicNone, LinearLODHeuristicSphere, PowHeuristicSphere}, palette::palette::MATERIAL_ID_BASE}};

        let scene = SceneWorker::new(&engine.context)?.run_worker(engine.context.clone(), 10);

        let palette = SharedPalette::new();
        let renderer = SceneRenderer::new(
            &engine.context, 
            &engine.swapchain, 
            &logic_state.camera,
            scene.render_data.clone(),
            palette.clone(),
            true,
        )?;

        let factor = 3.0;
        let mut csg = CSGTree::<u8, IVec3, i32, 3>::new_sphere_float(Vec3A::ZERO, 
            100.0 * factor, MATERIAL_ID_BASE);
        csg.cut_with_sphere(vec3a(70.0 * factor, 0.0, 0.0), 70.0 * factor, MATERIAL_ID_BASE);
        
        let now = Instant::now();

        let mut dag = VoxelDAG64::new(
            1000000, 
            64, 
            PowHeuristicSphere {
                center: (logic_state.camera.get_position_in_meters() * VOXELS_PER_METER as f32).as_ivec3(),
                render_dist: 15.0,
            }
        );
        let mut dag = dag.parallel();
        let key = dag.add_pos_query_volume(&csg)?;

        /*
        csg.cut_with_sphere(vec3a(70.0, 0.0, 0.0), 70.0, MATERIAL_ID_BASE);
        csg.calculate_bounds();

        let key = dag.update_pos_query_volume(&csg, key)?;
        */        

        let elapsed = now.elapsed();
        info!("Tree Build took {:.2?}", elapsed);

        let dag_key = scene.send.add_dag(dag.clone()).result_blocking();
        let object_key = scene.send.add_dag_object(
            Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                vec3(0.0, 0.0, 0.0)
            ),
            dag_key,
            dag.get_entry(key),
        ).result_blocking();
        
        Ok(RenderState {
            csg,
            scene,
            renderer,
            dag,
            dag_key,
            object_key
        })
    }

    #[cfg(feature="graph")]
    {
        use crate::mesh::scene::MeshScene;

        let palette = SharedPalette::new();
        let mesh_scene = MeshScene::new(&engine.context, &engine.swapchain);

        let mut scene = SceneRenderer::new(
            engine, 
            &logic_state.camera,
            palette.clone(),
            false,
        )?;

        let composer = ModelComposer::new(
            &logic_state.camera, 
            palette, 
            scene.worker_ref.send.clone(), 
            mesh_scene.send.to_owned()); 


        return Ok(RenderState {
            scene,
            composer,
            mesh_scene,
        })
    }

    #[cfg(not(any(feature="scene", feature="graph")))]
    {
        Ok(RenderState { })
    }
   }

#[unsafe(no_mangle)]
pub fn update(
    logic_state: &mut LogicState,
    render_state: &mut RenderState,
    engine: &mut Engine,
    delta_time: Duration,
) -> OctaResult<()> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    let time = logic_state.start_time.elapsed(); 

    logic_state.camera.update(&engine.controls, delta_time);
    //info!("Camera Pos: {} Dir: {}", logic_state.camera.get_position_in_meters(), logic_state.camera.direction);
    
    #[cfg(any(feature="scene"))]
    if engine.controls.f2 {
        use crate::voxel::dag64::lod_heuristic::{LinearLODHeuristicSphere, PowHeuristicSphere};

        render_state.scene.send.remove_object(render_state.object_key);
        
        render_state.dag.lod.center = (logic_state.camera.get_position_in_meters() * VOXELS_PER_METER as f32).as_ivec3();

        let key = render_state.dag.add_pos_query_volume(&render_state.csg)?;
        render_state.object_key = render_state.scene.send.add_dag_object(
            Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                vec3(0.0, 0.0, 0.0)
            ),
            render_state.dag_key,
            render_state.dag.get_entry(key),
        ).result_blocking();
    }

    #[cfg(any(feature="scene"))]
    render_state.renderer.update(
        &logic_state.camera, 
        &engine.context, 
        engine.get_resolution(), 
        engine.get_current_in_flight_frame_index(), 
        engine.get_current_frame_index())?;

    
    #[cfg(feature="graph")]
    {
        render_state.composer.update(time, &logic_state.camera)?;


        if render_state.composer.render_panel_changed {
            render_state.composer.render_panel_changed = false;

            debug!("Resize Render Panel");
            logic_state.camera.set_screen_size(render_state.composer.render_panel_size.as_vec2());

            render_state
                .scene
                .on_size_changed(
                    render_state.composer.render_panel_size,
                    &engine.context,
                    &engine.swapchain,
                )?;
        }

        render_state.mesh_scene.update(&engine.context);

        render_state.scene.update(
            engine,
            &logic_state.camera, 
            render_state.composer.render_panel_size)?;
    }

    Ok(())
}

#[unsafe(no_mangle)]
pub fn record_render_commands(
    logic_state: &mut LogicState,
    render_state: &mut RenderState,
    engine: &mut Engine,
) -> OctaResult<()> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    let command_buffer = engine.get_current_command_buffer();
    
    #[cfg(any(feature="scene"))]
    render_state.renderer.render(UVec2::ZERO, command_buffer, &engine, &logic_state.camera)?;

    #[cfg(any(feature="graph"))]
    render_state.scene.render(UVec2::ZERO, command_buffer, &engine, &logic_state.camera)?;
    
    #[cfg(any(feature="graph"))]
    render_state.mesh_scene.render(
        command_buffer, 
        &logic_state.camera,
        engine, 
        render_state.composer.render_panel_size.as_vec2(),
        &render_state.scene.voxel_renderer.palette_buffer,
    );
    // TODO: Move Palette Code


    #[cfg(not(any(feature="scene", feature="graph")))]
    command_buffer.swapchain_image_render_barrier(&engine.get_current_swapchain_image_and_view().image)?;

    Ok(())
}

#[unsafe(no_mangle)]
pub fn record_ui_commands(
    ctx: &egui::Context,
    logic_state: &mut LogicState,
    render_state: &mut RenderState,
) -> OctaResult<()> {
    #[cfg(any(feature="scene", feature="graph"))]
    render_state.scene.render_ui(ctx);

    #[cfg(any(feature="graph"))]
    render_state.composer.render(ctx);

    Ok(())
}

#[unsafe(no_mangle)]
pub fn on_window_event(
    _logic_state: &mut LogicState,
    render_state: &mut RenderState,
    engine: &mut Engine,
    event: &WindowEvent,
) -> OctaResult<()> {

    Ok(())
}

#[unsafe(no_mangle)]
pub fn on_recreate_swapchain(
    logic_state: &mut LogicState,
    render_state: &mut RenderState,
    engine: &mut Engine,
) -> OctaResult<()> {
    logic_state.camera.set_screen_size(engine.swapchain.size.as_vec2());

    #[cfg(any(feature="scene"))]
    render_state
        .renderer
            .on_size_changed(
                engine.swapchain.size,
                &engine.context,
                &engine.swapchain,
            )?;


    trace!("On recreate swapchain done");
    Ok(())
}
