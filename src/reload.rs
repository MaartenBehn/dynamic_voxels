mod render;
mod shader;

use std::time::{Duration, Instant};
use octa_force::{Engine, OctaResult};
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, Vec3};
use octa_force::log::Log;
use octa_force::logger::setup_logger;
use crate::render::renderer::Renderer;

pub struct RenderState {
    pub renderer: Renderer
}

pub struct LogicState {
    pub camera: Camera,
    pub start_time: Instant,
}

#[no_mangle]
pub fn init_hot_reload(logger: &'static dyn Log) -> OctaResult<()> {
    setup_logger(logger)?;

    Ok(())
}

#[no_mangle]
pub fn new_render_state(engine: &mut Engine) -> OctaResult<RenderState> {
    let renderer = Renderer::new(&engine.context, engine.swapchain.size, engine.num_frames)?;

    Ok(RenderState {
        renderer
    })
}

#[no_mangle]
pub fn new_logic_state(engine: &mut Engine) -> OctaResult<LogicState> {
    log::info!("Creating Camera");
    let mut camera = Camera::base(engine.swapchain.size.as_vec2());

    camera.position = Vec3::new(0.0, 0.0, 0.0);
    //camera.position = Vec3::new(1.0, -100.0, 1.0);
    camera.direction = Vec3::new(0.0, 1.0, 0.0).normalize();
    camera.speed = 10.0;
    camera.z_far = 100.0;
    camera.up = vec3(0.0, 0.0, 1.0);

    Ok(LogicState {
        camera,
        start_time: Instant::now(),
    })
}

#[no_mangle]
pub fn update(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine, image_index: usize, delta_time: Duration) -> OctaResult<()> {
    let time = logic_state.start_time.elapsed();
    
    logic_state.camera.update(&engine.controls, delta_time);
    render_state.renderer.update(&logic_state.camera, engine.swapchain.size, time)?;

    Ok(())
}

#[no_mangle]
pub fn record_render_commands(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine, image_index: usize) -> OctaResult<()> {

    let command_buffer = &engine.command_buffers[image_index];
    render_state.renderer.render(command_buffer, image_index, &engine.swapchain)?;

    Ok(())
}

#[no_mangle]
pub fn on_window_event(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine, event: &WindowEvent) -> OctaResult<()> {
    Ok(())
}

#[no_mangle]
pub fn on_recreate_swapchain(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine) -> OctaResult<()> {
    render_state.renderer
        .on_recreate_swapchain(&engine.context, engine.num_frames, engine.swapchain.size)?;

    Ok(())
}