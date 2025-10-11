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

use csg::csg_tree::tree::CSGTree;
use csg::union::tree::{Union, UnionNode};
use model::composer::ModelComposer;
use model::examples::compose_island::{ComposeIsland};
use octa_force::engine::Engine;
use parking_lot::Mutex;
use scene::dag64::SceneAddDAGObject;
use scene::renderer::SceneRenderer;
use scene::worker::SceneWorker;
use scene::{Scene, SceneObjectData, SceneObjectKey, SceneObjectType};
use slotmap::Key;
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, vec3a, DVec3, EulerRot, IVec2, IVec3, Mat4, Quat, UVec3, Vec3, Vec3A};
use octa_force::gui::Gui;
use octa_force::log::{debug, error, info, trace, Log};
use octa_force::logger::setup_logger;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use octa_force::vulkan::{Context, Fence, ImageBarrier};
use octa_force::{log, OctaResult};
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

pub const USE_PROFILE: bool = false;
pub const NUM_FRAMES_IN_FLIGHT: usize = 2;

pub const VOXELS_PER_METER: usize = 10;
pub const METERS_PER_SHADER_UNIT: usize = 1000;
pub const VOXELS_PER_SHADER_UNIT: usize = VOXELS_PER_METER * METERS_PER_SHADER_UNIT;

#[unsafe(no_mangle)]
pub fn init_hot_reload(logger: &'static dyn Log) -> OctaResult<()> {
    setup_logger(logger)?;

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
        camera.set_position_in_meters(Vec3::new(12.648946, 21.449644, 6.6995387)); 
        camera.direction = Vec3::new(-0.6110025, -0.7362994, -0.29075617).normalize();
        
        camera.speed = 10.0;
        camera.z_near = 0.001;
    }

    #[cfg(feature="graph_builder")]
    {
        camera.set_meter_per_unit(METERS_PER_SHADER_UNIT as f32);
        camera.set_position_in_meters(Vec3::new(0.0, -10.0, 2.0)); 
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
    pub gui: Gui,
         
    #[cfg(any(feature="scene", feature="islands", feature="graph_builder"))]
    pub scene: SceneWorker,
    
    #[cfg(any(feature="scene", feature="islands", feature="graph_builder"))]
    pub renderer: SceneRenderer,
    


    #[cfg(any(feature="islands"))]
    pub islands: ModelWorker<Islands>,

    #[cfg(any(feature="islands"))]
    pub template_debug: TemplateDebugGui<IslandGenerationTypes>,

    #[cfg(any(feature="islands"))]
    pub collapser_debug: CollapserDebugGui<IslandGenerationTypes>,

    #[cfg(feature="graph_builder")]
    pub islands: ComposeIsland
}

#[unsafe(no_mangle)]
pub fn new_render_state(logic_state: &mut LogicState, engine: &mut Engine) -> OctaResult<RenderState> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();
     
    let mut gui = Gui::new(
        &engine.context,
        engine.swapchain.format,
        engine.swapchain.depth_format,
        &engine.window,
        engine.in_flight_frames.num_frames,
    )?;
   
    #[cfg(feature="scene")]
    {
        let scene = Scene::new(&engine.context)?.run_worker(engine.context.get_alloc_context(), 10);

        let palette = SharedPalette::new();
        let renderer = SceneRenderer::new(
            &engine.context, 
            &engine.swapchain, 
            &logic_state.camera,
            scene.render_data.clone(),
            palette.clone(),
        )?;

        let mut csg = CSGTree::<u8, Int3D, 3>::new_sphere(Vec3A::ZERO, 100.0);
        csg.cut_with_sphere(vec3a(70.0, 0.0, 0.0), 70.0);
        
        let now = Instant::now();

        let mut dag = VoxelDAG64::new(1000000, 64);
        let mut dag = dag.parallel();
        let key = dag.add_aabb_query_volume(&csg)?;

       /* 
        let index = csg.append_node_with_remove(
            CSGNode::new_sphere(vec3(70.0, 0.0, 0.0), 50.0));
        csg.set_all_aabbs();

        let key = dag.update_aabb_query_volume(&csg, key)?;
*/

        let elapsed = now.elapsed();
        info!("Tree Build took {:.2?}", elapsed);

        let dag_key = scene.send.add_dag(dag.clone()).result_blocking();
        scene.send.add_dag_object(
            Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                vec3(0.0, 0.0, 0.0)
            ),
            dag_key,
            dag.get_entry(key),
        );
        
        Ok(RenderState {
            gui,
            scene,
            renderer,
        })
    }

    #[cfg(feature="graph_builder")]
    {
        let palette = SharedPalette::new();
        let scene = Scene::new(&engine.context)?.run_worker(engine.context.get_alloc_context(), 1000); 
        let islands = ComposeIsland::new(scene.send.to_owned()); 

        let mut renderer = SceneRenderer::new(
            &engine.context, 
            &engine.swapchain, 
            &logic_state.camera,
            scene.render_data.clone(),
            palette,
        )?;

        return Ok(RenderState {
            scene,
            renderer,
            gui, 
            islands,
        })
    }

    #[cfg(not(any(feature="scene", feature="graph_builder")))]
    {
        Ok(RenderState { gui })
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
    
    #[cfg(feature="graph_builder")]
    render_state.islands.update(time)?;

    #[cfg(any(feature="scene", feature="graph_builder"))]
    render_state.renderer.update(
        &logic_state.camera, 
        &engine.context, 
        engine.get_resolution(), 
        engine.get_current_in_flight_frame_index(), 
        engine.get_current_frame_index())?;

    
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
    
    #[cfg(any(feature="scene", feature="graph_builder"))]
    render_state.renderer.render(command_buffer, &engine)?;

    #[cfg(not(any(feature="scene", feature="graph_builder")))]
    command_buffer.swapchain_image_render_barrier(&engine.get_current_swapchain_image_and_view().image)?;

    command_buffer.begin_rendering(
        &engine.get_current_swapchain_image_and_view().view,
        &engine.get_current_depth_image_and_view().view,
        engine.swapchain.size,
        AttachmentLoadOp::DONT_CARE,
        None,
    );

    render_state.gui.cmd_draw(
        command_buffer,
        engine.get_resolution(),
        engine.get_current_in_flight_frame_index(),
        &engine.window,
        &engine.context,
        |ctx| {
             
            #[cfg(any(feature="scene", feature="graph_builder"))]
            render_state.renderer.render_ui(ctx);

            #[cfg(any(feature="graph_builder"))]
            render_state.islands.render(ctx);
        },
    )?;

    command_buffer.end_rendering();

    Ok(())
}

#[unsafe(no_mangle)]
pub fn on_window_event(
    _logic_state: &mut LogicState,
    render_state: &mut RenderState,
    engine: &mut Engine,
    event: &WindowEvent,
) -> OctaResult<()> {
    render_state.gui.handle_event(&engine.window, event);

    Ok(())
}

#[unsafe(no_mangle)]
pub fn on_recreate_swapchain(
    logic_state: &mut LogicState,
    render_state: &mut RenderState,
    engine: &mut Engine,
) -> OctaResult<()> {
    logic_state.camera.set_screen_size(engine.swapchain.size.as_vec2());

    #[cfg(any(feature="scene", feature="graph_builder"))]
    render_state
        .renderer
            .on_recreate_swapchain(
                &engine.context,
                &engine.swapchain,
            )?;

    trace!("On recreate swapchain done");
    Ok(())
}
