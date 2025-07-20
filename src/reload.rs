#![forbid(unused_must_use)]
extern crate self as dynamic_voxels;

pub mod csg;
pub mod model;
pub mod multi_data_buffer;
pub mod scene;
pub mod util;
pub mod volume;
pub mod voxel;

use csg::fast_query_csg_tree::tree::FastQueryCSGTree;
use csg::slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGTree};
use csg::vec_csg_tree::tree::{VecCSGTree, VOXEL_SIZE};
use model::debug_renderer::ModelDebugRenderer;
use model::examples::islands::{self, IslandsState};
use octa_force::engine::Engine;
use parking_lot::Mutex;
use scene::dag64::DAG64SceneObject;
use scene::renderer::SceneRenderer;
use scene::{Scene, SceneObjectData, SceneObjectKey, SceneObjectType};
use slotmap::Key;
use kiddo::SquaredEuclidean;
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, DVec3, EulerRot, Mat4, Quat, UVec3, Vec3};
use octa_force::gui::Gui;
use octa_force::log::{debug, error, info, Log};
use octa_force::logger::setup_logger;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use octa_force::vulkan::{Context, Fence};
use octa_force::{log, OctaResult};
use util::profiler::ShaderProfiler;
use util::state_saver::StateSaver;
use volume::VolumeBounds;
use voxel::dag64::VoxelDAG64;
use voxel::grid::VoxelGrid;
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
        camera.set_position_in_meters(Vec3::new(15.501491, -12.135776, 3.5071237)); 
        camera.direction = Vec3::new(-0.580947, 0.7954067, -0.17271005).normalize();
        
        camera.speed = 10.0;
        camera.z_near = 0.001;
    }

    #[cfg(feature="islands")]
    {
        camera.set_meter_per_unit(METERS_PER_SHADER_UNIT as f32);
        camera.set_position_in_meters(Vec3::new(0.0, -4.0, 0.0)); 
        camera.direction = Vec3::new(0.0, 1.0, 0.0).normalize();
        
        camera.speed = 10.0;
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
         
    #[cfg(any(feature="scene", feature="islands"))]
    pub renderer: SceneRenderer,
    
    #[cfg(any(feature="islands"))]
    pub state_saver: StateSaver<IslandsState>,
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
        let mut scene = Scene::new(&engine.context)?;

        let mut csg = SlotMapCSGTree::new_sphere(Vec3::ZERO, 100.0);

        let now = Instant::now();

        let mut tree64 = VoxelDAG64::from_aabb_query(&csg)?;

        let index = csg.append_node_with_remove(
            SlotMapCSGNode::new_sphere(vec3(70.0, 0.0, 0.0), 50.0));
        csg.set_all_aabbs();
        let aabb = csg.nodes[index].aabb;

        let key = tree64.update_aabb(&csg, aabb, tree64.get_first_key())?;

        let elapsed = now.elapsed();
        info!("Tree Build took {:.2?}", elapsed);

        scene.add_dag64(
            &engine.context,
            Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                vec3(0.0, 0.0, 0.0)
            ), 
            key,
            Arc::new(Mutex::new(tree64)),
        )?;

        let renderer = SceneRenderer::new(&engine.context, &engine.swapchain, scene, &logic_state.camera)?;

        Ok(RenderState {
            gui,
            renderer,
        })
    }

    #[cfg(feature="islands")]
    {
        let state = IslandsState::new(false)?;
        
        let mut scene = Scene::new(&engine.context)?; 
        let renderer = SceneRenderer::new(&engine.context, &engine.swapchain, scene, &logic_state.camera)?;

        let state_saver = StateSaver::from_state(state, 10);
        
        Ok(RenderState {
            gui,
            renderer,
            state_saver,
        })
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
    
    #[cfg(any(feature="islands"))]
    if render_state.state_saver.tick(&mut |s: &mut IslandsState| -> OctaResult<bool> {
        s.update(&logic_state.camera)?;
        s.tick(&mut render_state.renderer.scene, &engine.context)
    })? {

    }

    #[cfg(any(feature="scene", feature="islands"))]
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
    
    #[cfg(any(feature="scene", feature="islands"))]
    render_state.renderer.render(command_buffer, &engine)?;

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
             
            #[cfg(any(feature="scene"))]
            render_state.renderer.render_ui(ctx);

            #[cfg(any(feature="islands"))]
            render_state.state_saver.render(ctx);
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

    #[cfg(any(feature="scene", feature="islands"))]
    render_state
        .renderer
            .on_recreate_swapchain(
                &engine.context,
                &engine.swapchain,
            )?;

    Ok(())
}
