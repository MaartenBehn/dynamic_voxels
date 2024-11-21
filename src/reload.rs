use std::time::Duration;
use octa_force::{Engine, OctaResult};
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::gui::Gui;
use octa_force::log::Log;
use octa_force::logger::setup_logger;
use crate::visualization::Visualization;

mod util;
mod render;
mod grid;
mod rules;
mod visualization;
mod grid_manager;
mod value;
mod rule_gen;

pub struct RenderState {
    visualization: Visualization,
    gui: Gui,
}

pub struct LogicState {
    
}

#[no_mangle]
pub fn init_hot_reload(logger: &'static dyn Log) -> OctaResult<()> {
    setup_logger(logger)?;

    Ok(())
}

#[no_mangle]
pub fn new_render_state(engine: &mut Engine) -> OctaResult<RenderState> {
    let visualization = Visualization::new(engine)?;
    
    let gui = Gui::new(
        &engine.context,
        engine.swapchain.format,
        engine.swapchain.depth_format,
        &engine.window,
        engine.num_frames
    )?;

    
    
    Ok(RenderState {
        visualization,
        gui,
    })
}

#[no_mangle]
pub fn new_logic_state(_: &mut Engine) -> OctaResult<LogicState> {
    Ok(LogicState {
        
    })
}

#[no_mangle]
pub fn update(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine, image_index: usize, delta_time: Duration) -> OctaResult<()> {
    render_state.visualization.update(engine, image_index, delta_time)
}

#[no_mangle]
pub fn record_render_commands(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine, image_index: usize) -> OctaResult<()> {
    render_state.visualization.record_render_commands(engine, image_index)
}

#[no_mangle]
pub fn on_window_event(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine, event: &WindowEvent) -> OctaResult<()> {
    render_state.visualization.on_window_event(engine, event)
}

#[no_mangle]
pub fn on_recreate_swapchain(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine) -> OctaResult<()> {
    Ok(())
}