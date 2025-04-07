#![forbid(unused_must_use)]

mod aabb;
mod buddy_controller;
mod vec_csg_tree;
mod profiler;
mod util;
mod model_synthesis;
mod voxel;
mod volume;
mod csg_renderer;
mod render_csg_tree;
mod slot_map_csg_tree;
mod model_renderer;
mod model_example;
mod state_saver;

use crate::vec_csg_tree::tree::{VecCSGTree, VOXEL_SIZE};
use crate::profiler::ShaderProfiler;
use csg_renderer::color_controller::ColorController;
use csg_renderer::data_controller::DataController;
use csg_renderer::Renderer;
use model_example::fence::FenceState;
use model_renderer::ModelDebugRenderer;
use model_synthesis::collapser_data::CollapserData;
use model_synthesis::template::TemplateTree;
use octa_force::engine::Engine;
use render_csg_tree::base::RenderCSGTree;
use slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};
use slotmap::Key;
use state_saver::StateSaver;
use vec_csg_tree::tree::{VecCSGNode, VecCSGNodeData};
use kiddo::SquaredEuclidean;
use model_synthesis::collapse::{CollapseOperation, Collapser};
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, Mat4, Quat, Vec3};
use octa_force::gui::Gui;
use octa_force::log::{debug, error, info, Log};
use octa_force::logger::setup_logger;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use octa_force::vulkan::Fence;
use octa_force::{log, OctaResult};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use std::default;
use model_synthesis::builder::{BuilderAmmount, ModelSynthesisBuilder, IT};

pub const USE_PROFILE: bool = false;


#[unsafe(no_mangle)]
pub fn init_hot_reload(logger: &'static dyn Log) -> OctaResult<()> {
    setup_logger(logger)?;

    Ok(())
}

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

    camera.position = Vec3::new(1.0, -10.0, 1.0);
    camera.direction = Vec3::new(0.1, 1.0, 0.0).normalize();
    // camera.position = Vec3::new(67.02305, 127.88921, 43.476604);
    // camera.direction = Vec3::new(0.79322153, -0.47346807, -0.38291982).normalize();
    camera.speed = 10.0 * VOXEL_SIZE;
    camera.z_far = 100.0;
    camera.up = vec3(0.0, 0.0, 1.0);

    Ok(LogicState {
        camera,
        start_time: Instant::now(),
    })
}


pub struct RenderState {
    pub gui: Gui,
    pub data_controller: DataController,
    pub color_controller: ColorController,
    pub renderer: Renderer,
    pub profiler: Option<ShaderProfiler>,

    pub state_saver: StateSaver<FenceState>,
    pub model_renderer: ModelDebugRenderer,
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
        engine.num_frames,
    )?;

    let data_controller = DataController::new(&engine.context)?;

    let color_controller = ColorController::new(&engine.context)?;

    let profiler = if engine.context.shader_clock {
        Some(ShaderProfiler::new(
            &engine.context,
            engine.swapchain.format,
            engine.swapchain.size,
            engine.num_frames,
            profile_scopes,
            &mut gui.renderer,
        )?)
    } else {
        None
    };

    let renderer = Renderer::new(
        &engine.context,
        engine.swapchain.size,
        engine.num_frames,
        &data_controller,
        &color_controller,
        &profiler,
        shader_bin,
    )?;

    // let mut tree = CSGTree::new_example_tree_2(1.0);
    // csg_controller.set_data(&tree.make_data());

    let fence_state = FenceState::new(); 
    let state_saver = StateSaver::from_state(fence_state, 10);

    let mut model_renderer = ModelDebugRenderer::default();
 
    Ok(RenderState {
        gui,
        data_controller,
        color_controller,
        renderer,
        profiler,

        state_saver,
        model_renderer,
    })
}




#[unsafe(no_mangle)]
pub fn update(
    logic_state: &mut LogicState,
    render_state: &mut RenderState,
    engine: &mut Engine,
    frame_index: usize,
    delta_time: Duration,
) -> OctaResult<()> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    let time = logic_state.start_time.elapsed(); 

    logic_state.camera.update(&engine.controls, delta_time);
    // info!("Camera Pos: {} Dir: {}", logic_state.camera.position, logic_state.camera.direction);

    render_state
        .renderer
        .update(&logic_state.camera, engine.swapchain.size, time)?;

    if render_state.profiler.is_some() {
        render_state
            .profiler
            .as_mut()
            .unwrap()
            .update(frame_index, &engine.context)?;
    }

    if render_state.state_saver.tick()? {
        let state = render_state.state_saver.get_state_mut();
        render_state.model_renderer.update(&mut state.collapser.as_mut().unwrap());

        if let Some(csg) = state.csg.clone() {
            let vec_tree: VecCSGTree = csg.into();
            render_state.data_controller.set_render_csg_tree(&vec_tree.into())?;    
        }
    }

    Ok(())
}

#[unsafe(no_mangle)]
pub fn record_render_commands(
    _logic_state: &mut LogicState,
    render_state: &mut RenderState,
    engine: &mut Engine,
    image_index: usize,
) -> OctaResult<()> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    let command_buffer = &engine.command_buffers[image_index];
    render_state
        .renderer
        .render(command_buffer, image_index, &engine.swapchain)?;

    command_buffer.begin_rendering(
        &engine.swapchain.images_and_views[image_index].view,
        &engine.swapchain.depht_images_and_views[image_index].view,
        engine.swapchain.size,
        AttachmentLoadOp::DONT_CARE,
        None,
    );

    render_state.gui.cmd_draw(
        command_buffer,
        engine.swapchain.size,
        image_index,
        &engine.window,
        &engine.context,
        |ctx| {
            if render_state.profiler.is_some() {
                render_state
                    .profiler
                    .as_mut()
                    .unwrap()
                    .gui_windows(ctx, engine.controls.mouse_left);
            }

            render_state.state_saver.render(ctx);

            let state = render_state.state_saver.get_state_mut();
            render_state.model_renderer.render(ctx, state.collapser.as_mut().unwrap());
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
    
    render_state.renderer.on_recreate_swapchain(
        &engine.context,
        engine.num_frames,
        engine.swapchain.size,
    )?;

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

    Ok(())
}
