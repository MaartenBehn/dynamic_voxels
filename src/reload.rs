#![forbid(unused_must_use)]
extern crate self as dynamic_voxels;

pub mod aabb;
pub mod buddy_controller;
pub mod vec_csg_tree;
pub mod profiler;
pub mod util;
pub mod model_synthesis;
pub mod voxel_grid;
pub mod volume;
pub mod csg_renderer;
pub mod render_csg_tree;
pub mod slot_map_csg_tree;
pub mod model_debug_renderer;
pub mod model_example;
pub mod state_saver;
pub mod voxel_tree64;
pub mod voxel_renderer;
pub mod scene;

use crate::vec_csg_tree::tree::{VecCSGTree, VOXEL_SIZE};
use crate::profiler::ShaderProfiler;
use csg_renderer::color_controller::ColorController;
use csg_renderer::data_controller::DataController;
use csg_renderer::CSGRenderer;
use model_debug_renderer::ModelDebugRenderer;
use model_synthesis::collapser_data::CollapserData;
use model_synthesis::template::TemplateTree;
use octa_force::engine::Engine;
use render_csg_tree::base::RenderCSGTree;
use scene::renderer::SceneRenderer;
use scene::tree64::Tree64SceneObject;
use scene::{Scene, SceneObject};
use slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};
use slotmap::Key;
use state_saver::StateSaver;
use vec_csg_tree::tree::{VecCSGNode, VecCSGNodeData};
use kiddo::SquaredEuclidean;
use model_synthesis::collapse::{CollapseOperation, Collapser};
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, Mat4, Quat, UVec3, Vec3};
use octa_force::gui::Gui;
use octa_force::log::{debug, error, info, Log};
use octa_force::logger::setup_logger;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use octa_force::vulkan::Fence;
use octa_force::{log, OctaResult};
use voxel_grid::VoxelGrid;
use voxel_tree64::renderer::Tree64Renderer;
use voxel_tree64::VoxelTree64;
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use std::{default, env};
use model_synthesis::builder::{BuilderAmmount, ModelSynthesisBuilder, IT};

use model_example::fence::{self, FenceState};
use model_example::islands::{self, IslandsState};

pub const USE_PROFILE: bool = false;
pub const NUM_FRAMES_IN_FLIGHT: usize = 2;

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

    #[cfg(feature="fence")]
    {
        camera.position = Vec3::new(1.0, -10.0, 1.0); 
        camera.direction = Vec3::new(0.1, 1.0, 0.0).normalize();
        camera.speed = 10.0 * VOXEL_SIZE;
    }

    #[cfg(feature="islands")]
    {
        camera.position = Vec3::new(1.0, -10.0, 1.0); 
        camera.direction = Vec3::new(0.1, 1.0, 0.0).normalize();
        camera.speed = 10.0 * VOXEL_SIZE;
    }
    
    #[cfg(feature="render_example")]
    {
        camera.position = Vec3::new(67.02305, 127.88921, 43.476604);
        camera.direction = Vec3::new(0.79322153, -0.47346807, -0.38291982).normalize();
        camera.speed = 10.0 * VOXEL_SIZE;
    }

     #[cfg(feature="tree64")]
    {
        camera.position = Vec3::new(-0.66225964, -0.10641506, 0.803499); 
        camera.direction = Vec3::new(0.87766635, 0.373136, -0.30078444).normalize();        
        camera.position = Vec3::new(0.2, -2.0, 1.0); 
        camera.direction = Vec3::new(0.1, 1.0, -0.5).normalize();

        camera.speed = 1.0;
        camera.z_near = 0.001;
    }

    #[cfg(feature="scene")]
    {
        camera.position = Vec3::new(0.2, -2.0, 1.0); 
        camera.direction = Vec3::new(0.1, 1.0, -0.5).normalize();
        
        camera.speed = 1.0;
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
     
    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    pub data_controller: DataController,
    
    #[cfg(any(feature="fence",feature="islands", feature="render_example"))]
    pub color_controller: ColorController,
    
    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    pub renderer: CSGRenderer,

    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    pub profiler: Option<ShaderProfiler>,

    #[cfg(feature="fence")]
    pub state_saver: StateSaver<FenceState>,
   
    #[cfg(feature="islands")]
    pub state_saver: StateSaver<IslandsState>,
    
    #[cfg(feature="fence")]
    pub model_renderer: ModelDebugRenderer<fence::Identifier>,
     
    #[cfg(feature="islands")]
    pub model_renderer: ModelDebugRenderer<islands::Identifier>,
    
    #[cfg(feature="tree64")]
    pub renderer: Tree64Renderer,
    
    #[cfg(feature="scene")]
    pub renderer: SceneRenderer
}

#[unsafe(no_mangle)]
pub fn new_render_state(logic_state: &mut LogicState, engine: &mut Engine) -> OctaResult<RenderState> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();
    
    let (shader_bin, profile_scopes): (&[u8], &[&str]) = 
        if USE_PROFILE && engine.context.shader_clock {
            shaders::trace_ray_profile()
        } else {
            shaders::trace_ray()
        };

    let mut gui = Gui::new(
        &engine.context,
        engine.swapchain.format,
        engine.swapchain.depth_format,
        &engine.window,
        engine.in_flight_frames.num_frames,
    )?;

    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    let data_controller = DataController::new(&engine.context)?;

    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    let color_controller = ColorController::new(&engine.context)?;

    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    let profiler = if engine.context.shader_clock {
        Some(ShaderProfiler::new(
            &engine.context,
            engine.swapchain.format,
            engine.get_resolution(),
            engine.get_num_frames_in_flight(),
            profile_scopes,
            &mut gui.renderer,
        )?)
    } else {
        None
    };

    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    let renderer = CSGRenderer::new(
        &engine.context,
        engine.get_resolution(),
        engine.get_num_frames_in_flight(),
        &data_controller,
        &color_controller,
        &profiler,
        shader_bin,
    )?;

    
    #[cfg(feature="render_example")]
    let mut tree = VecCSGTree::new_example_tree_2(1.0);
    #[cfg(feature="render_example")]
    data_controller.set_render_csg_tree(&tree.into())?;
   

    #[cfg(feature="fence")]
    let fence_state = FenceState::new(); 
    
    #[cfg(feature="fence")]
    let state_saver = StateSaver::from_state(fence_state, 10);

    #[cfg(feature="fence")]
    let mut model_renderer = ModelDebugRenderer::default();


    #[cfg(feature="islands")]
    let island_state = IslandsState::new(); 
    
    #[cfg(feature="islands")]
    let state_saver = StateSaver::from_state(island_state, 10);

    #[cfg(feature="islands")]
    let mut model_renderer = ModelDebugRenderer::default();


    #[cfg(any(feature="tree64", feature="scene"))]
    let mut grid = VoxelGrid::new(UVec3::ONE * 4_u32.pow(2));
     
    #[cfg(any(feature="tree64", feature="scene"))]
    grid.set_example_sphere();
    
    #[cfg(any(feature="tree64", feature="scene"))]
    grid.set_corners();

    #[cfg(any(feature="tree64", feature="scene"))]
    let tree64: VoxelTree64 = grid.into();
    
    #[cfg(feature="tree64")]
    let tree_renderer = Tree64Renderer::new(&engine.context, &engine.swapchain, tree64, &logic_state.camera)?;

    #[cfg(feature="scene")]
    let mut scene = Scene::from_objects(&engine.context, vec![
        SceneObject::Tree64(Tree64SceneObject::new(Mat4::from_euler(octa_force::glam::EulerRot::XZY, 0.0, 0.0, f32::to_radians(45.0)), tree64))
    ])?;

    #[cfg(feature="scene")]
    let scene_renderer = SceneRenderer::new(&engine.context, &engine.swapchain, scene, &logic_state.camera)?;

    Ok(RenderState {
        gui,
        
        #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
        data_controller,
        
        #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
        color_controller,
        
        #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
        renderer,
        
        #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
        profiler,

        #[cfg(any(feature="fence", feature="islands"))]
        state_saver,

        #[cfg(any(feature="fence", feature="islands"))]
        model_renderer,

        #[cfg(feature="tree64")]
        renderer: tree_renderer,

        #[cfg(feature="scene")]
        renderer: scene_renderer,
    })
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
    //info!("Camera Pos: {} Dir: {}", logic_state.camera.position, logic_state.camera.direction);

    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    render_state
        .renderer
        .update(&logic_state.camera, engine.swapchain.size, time)?;

    #[cfg(any(feature="fence", feature="islands"))]
    render_state.model_renderer.update_show(&engine.controls);

    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    if render_state.profiler.is_some() {
        render_state
            .profiler
            .as_mut()
            .unwrap()
            .update(engine.get_current_in_flight_frame_index(), &engine.context)?;
    }

    #[cfg(any(feature="fence", feature="islands"))]
    if render_state.state_saver.tick()? {
        let state = render_state.state_saver.get_state_mut();
        render_state.model_renderer.update(&mut state.collapser.as_mut().unwrap());

        #[cfg(any(feature="fence"))]
        if let Some(csg) = state.csg.clone() {
            let vec_tree: VecCSGTree = csg.into();
            render_state.data_controller.set_render_csg_tree(&vec_tree.into())?;    
        }
    }

    #[cfg(any(feature="tree64", feature="scene"))]
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
    
    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    render_state
        .renderer
        .render(command_buffer, &engine)?;


    #[cfg(any(feature="tree64", feature="scene"))]
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
            
            #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
            if render_state.profiler.is_some() {
                render_state
                    .profiler
                    .as_mut()
                    .unwrap()
                    .gui_windows(ctx, engine.controls.mouse_left);
            }

            #[cfg(any(feature="fence", feature="islands"))]
            render_state.state_saver.render(ctx);

            #[cfg(any(feature="fence", feature="islands"))]
            let state = render_state.state_saver.get_state_mut();
             
            #[cfg(any(feature="fence", feature="islands"))]
            render_state.model_renderer.render(ctx, state.collapser.as_mut().unwrap());

            #[cfg(any(feature="tree64", feature="scene"))]
            render_state.renderer.render_ui(ctx);
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
    
    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    render_state.renderer.on_recreate_swapchain(
        &engine.context,
        engine.get_num_frames_in_flight(),
        engine.swapchain.size,
    )?;

    #[cfg(any(feature="fence", feature="islands", feature="render_example"))]
    if render_state.profiler.is_some() {
        render_state
            .profiler
            .as_mut()
            .unwrap()
            .on_recreate_swapchain(
                &engine.context,
                engine.swapchain.format,
                engine.swapchain.size,
            )?;
    }

    #[cfg(any(feature="tree64", feature="scene"))]
    render_state
        .renderer
            .on_recreate_swapchain(
                &engine.context,
                &engine.swapchain,
            )?;

    Ok(())
}
