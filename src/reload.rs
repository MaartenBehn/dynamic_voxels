mod render;
mod shader;
mod cgs_tree;
mod profiler;

use std::time::{Duration, Instant};
use log::debug;
use octa_force::{Engine, OctaResult};
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, Vec3};
use octa_force::log::Log;
use octa_force::logger::setup_logger;
use crate::cgs_tree::CGSTree;
use crate::profiler::Profiler;
use crate::render::renderer::Renderer;

pub struct RenderState {
    pub renderer: Renderer,
    pub profiler: Profiler,
}

pub struct LogicState {
    pub camera: Camera,
    pub start_time: Instant,
    pub cgs_tree: CGSTree,
}

#[no_mangle]
pub fn init_hot_reload(logger: &'static dyn Log) -> OctaResult<()> {
    setup_logger(logger)?;

    Ok(())
}

#[no_mangle]
pub fn new_render_state(engine: &mut Engine) -> OctaResult<RenderState> {
    let profiler = Profiler::new(&engine.context, engine.swapchain.size, engine.num_frames)?;
    let renderer = Renderer::new(&engine.context, engine.swapchain.size, engine.num_frames, &profiler)?;

    Ok(RenderState {
        renderer,
        profiler,
    })
}

#[no_mangle]
pub fn new_logic_state(render_state: &mut RenderState, engine: &mut Engine) -> OctaResult<LogicState> {
    let mut cgs_tree = CGSTree::new();
    cgs_tree.set_example_tree();
    cgs_tree.make_data();

    render_state.renderer.set_cgs_tree(&cgs_tree.data)?;
    
    log::info!("Creating Camera");
    let mut camera = Camera::base(engine.swapchain.size.as_vec2());

    camera.position = Vec3::new(-2.0, -5.0, 0.0);
    //camera.position = Vec3::new(1.0, -100.0, 1.0);
    //camera.direction = Vec3::new(0.1, 1.0, 0.0).normalize();
    camera.direction = Vec3::new(0.8260885, -0.14534459, -0.5444748).normalize();
    camera.speed = 10.0;
    camera.z_far = 100.0;
    camera.up = vec3(0.0, 0.0, 1.0);

    Ok(LogicState {
        camera,
        start_time: Instant::now(),
        cgs_tree,
    })
}

#[no_mangle]
pub fn update(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine, _image_index: usize, delta_time: Duration) -> OctaResult<()> {
    let time = logic_state.start_time.elapsed();
    
    logic_state.camera.update(&engine.controls, delta_time);
    render_state.renderer.update(&logic_state.camera, engine.swapchain.size, time)?;
    
    //debug!("{:?}", logic_state.camera.direction);
    
    render_state.profiler.print_result()?;

    Ok(())
}

#[no_mangle]
pub fn record_render_commands(render_state: &mut RenderState, _logic_state: &mut LogicState, engine: &mut Engine, image_index: usize) -> OctaResult<()> {

    let command_buffer = &engine.command_buffers[image_index];
    render_state.renderer.render(command_buffer, image_index, &engine.swapchain)?;

    Ok(())
}

#[no_mangle]
pub fn on_window_event(_render_state: &mut RenderState, _logic_state: &mut LogicState, _engine: &mut Engine, _event: &WindowEvent) -> OctaResult<()> {
    Ok(())
}

#[no_mangle]
pub fn on_recreate_swapchain(render_state: &mut RenderState, _logic_state: &mut LogicState, engine: &mut Engine) -> OctaResult<()> {
    render_state.renderer
        .on_recreate_swapchain(&engine.context, engine.num_frames, engine.swapchain.size)?;

    Ok(())
}