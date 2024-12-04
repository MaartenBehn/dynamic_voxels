mod render;
mod cgs_tree;
mod profiler;
mod aabb;

use std::time::{Duration, Instant};
use log::debug;
use octa_force::{egui, Engine, OctaResult};
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, Vec3};
use octa_force::gui::Gui;
use octa_force::log::Log;
use octa_force::logger::setup_logger;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use glsl_compiler::glsl;
use crate::cgs_tree::CGSTree;
use crate::profiler::ShaderProfiler;
use crate::render::renderer::Renderer;

pub struct RenderState {
    pub gui: Gui,
    pub renderer: Renderer,
    pub profiler: Option<ShaderProfiler>,
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
pub fn new_render_state(engine: &mut Engine, ) -> OctaResult<RenderState> {

    let (shader_bin, profile_scopes): (&[u8], &[&str]) = if engine.context.shader_clock {
        glsl!{type = Compute, profile, file = "shaders/trace_ray.comp"}
    } else {
        glsl!{type = Compute, file = "shaders/trace_ray.comp"}
    };
    

    let mut gui = Gui::new(&engine.context, engine.swapchain.format,  engine.swapchain.depth_format, &engine.window, engine.num_frames)?;

    let profiler = if engine.context.shader_clock {
        Some(ShaderProfiler::new(&engine.context, engine.swapchain.format, engine.swapchain.size, engine.num_frames, profile_scopes, &mut gui.renderer)?)
    } else {
        None
    };
    
    let renderer = Renderer::new(&engine.context, engine.swapchain.size, engine.num_frames, &profiler, shader_bin)?;

    Ok(RenderState {
        gui,
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
pub fn update(render_state: &mut RenderState, logic_state: &mut LogicState, engine: &mut Engine, frame_index: usize, delta_time: Duration) -> OctaResult<()> {
    let time = logic_state.start_time.elapsed();
    
    logic_state.camera.update(&engine.controls, delta_time);
    render_state.renderer.update(&logic_state.camera, engine.swapchain.size, time)?;
    //debug!("{:?}", logic_state.camera.direction);
    
    if render_state.profiler.is_some() {
        render_state.profiler.as_mut().unwrap().update(frame_index, &engine.context)?;
    }

    Ok(())
}

#[no_mangle]
pub fn record_render_commands(render_state: &mut RenderState, _logic_state: &mut LogicState, engine: &mut Engine, image_index: usize) -> OctaResult<()> {

    let command_buffer = &engine.command_buffers[image_index];
    render_state.renderer.render(command_buffer, image_index, &engine.swapchain)?;

    command_buffer.begin_rendering(
        &engine.swapchain.images_and_views[image_index].view,
        &engine.swapchain.depht_images_and_views[image_index].view,
        engine.swapchain.size,
        AttachmentLoadOp::DONT_CARE,
        None,
    );

    render_state.gui.cmd_draw(command_buffer, engine.swapchain.size, image_index, &engine.window, &engine.context, |ctx| {
        
        if render_state.profiler.is_some() {
            render_state.profiler.as_mut().unwrap().gui_windows(ctx, engine.controls.mouse_left);
        }
        
    })?;

    command_buffer.end_rendering();

    Ok(())
}

#[no_mangle]
pub fn on_window_event(render_state: &mut RenderState, _logic_state: &mut LogicState, engine: &mut Engine, event: &WindowEvent) -> OctaResult<()> {
    render_state.gui.handle_event(&engine.window, event);
    
    Ok(())
}

#[no_mangle]
pub fn on_recreate_swapchain(render_state: &mut RenderState, _logic_state: &mut LogicState, engine: &mut Engine) -> OctaResult<()> {
    render_state.renderer
        .on_recreate_swapchain(&engine.context, engine.num_frames, engine.swapchain.size)?;

    if render_state.profiler.is_some() {
        render_state.profiler.as_mut().unwrap().on_recreate_swapchain(&engine.context, engine.swapchain.format, engine.swapchain.size)?;
    }

    Ok(())
}