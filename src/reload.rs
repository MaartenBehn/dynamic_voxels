mod aabb;
mod buddy_controller;
mod cgs_tree;
mod color;
mod material;
mod profiler;
mod util;
mod wfc;

use crate::cgs_tree::controller::CSGController;
use crate::cgs_tree::tree::{CSGTree, VOXEL_SIZE};
use crate::color::ColorController;
use crate::material::controller::MaterialController;
use crate::material::voxels::VoxelField;
use crate::profiler::ShaderProfiler;
use cgs_tree::renderer::Renderer;
use cgs_tree::tree::{CSGNode, CSGNodeData, MATERIAL_NONE};
use glsl_compiler::glsl;
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, Mat4, Quat, Vec3};
use octa_force::gui::Gui;
use octa_force::log::Log;
use octa_force::logger::setup_logger;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use octa_force::{log, Engine, OctaResult};
use std::time::{Duration, Instant};
use wfc::builder::{NumberRangeDefinesType, WFCBuilder};
use wfc::node::{Node, WFC};
use wfc::renderer::renderer::WFCRenderer;

pub const USE_PROFILE: bool = false;

pub struct RenderState {
    pub gui: Gui,
    pub csg_controller: CSGController,
    pub material_controller: MaterialController,
    pub voxel_field: VoxelField,
    pub color_controller: ColorController,
    pub renderer: Renderer,
    pub profiler: Option<ShaderProfiler>,
    pub wfc_renderer: WFCRenderer,
}

pub struct LogicState {
    pub camera: Camera,
    pub start_time: Instant,
    pub wfc: WFC<()>,
}

#[no_mangle]
pub fn init_hot_reload(logger: &'static dyn Log) -> OctaResult<()> {
    setup_logger(logger)?;

    Ok(())
}

#[no_mangle]
pub fn new_render_state(engine: &mut Engine) -> OctaResult<RenderState> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    let (shader_bin, profile_scopes): (&[u8], &[&str]) =
        if USE_PROFILE && engine.context.shader_clock {
            glsl! {type = Compute, profile, file = "shaders/trace_ray.comp"}
        } else {
            glsl! {type = Compute, file = "shaders/trace_ray.comp"}
        };

    let mut gui = Gui::new(
        &engine.context,
        engine.swapchain.format,
        engine.swapchain.depth_format,
        &engine.window,
        engine.num_frames,
    )?;

    let csg_controller = CSGController::new(&engine.context)?;
    let mut material_controller = MaterialController::new(&engine.context)?;

    let mut voxel_field = VoxelField::new(16);
    voxel_field.set_example_sphere();
    material_controller.allocate_voxel_field(&mut voxel_field)?;
    material_controller.push_voxel_field(&voxel_field)?;

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
        &csg_controller,
        &material_controller,
        &color_controller,
        &profiler,
        shader_bin,
    )?;

    let wfc_renderer = WFCRenderer::new();

    Ok(RenderState {
        gui,
        csg_controller,
        material_controller,
        voxel_field,
        color_controller,
        renderer,
        profiler,
        wfc_renderer,
    })
}

#[no_mangle]
pub fn new_logic_state(
    render_state: &mut RenderState,
    engine: &mut Engine,
) -> OctaResult<LogicState> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    log::info!("Creating Camera");
    let mut camera = Camera::base(engine.swapchain.size.as_vec2());

    camera.position = Vec3::new(1.0, -10.0, 1.0);
    camera.direction = Vec3::new(0.1, 1.0, 0.0).normalize();
    // camera.position = Vec3::new(-37.049347, -44.29117, 6.102236);
    // camera.direction = Vec3::new(0.7462443, -0.059647024, -0.6629945).normalize();
    camera.speed = 10.0 * VOXEL_SIZE;
    camera.z_far = 100.0;
    camera.up = vec3(0.0, 0.0, 1.0);

    let wfc_builder = WFCBuilder::new()
        .node(|b| {
            b.identifier(0)
                // Number of fence posts
                .number_range(2..=5, |b| {
                    b.identifier(1)
                        .defines(NumberRangeDefinesType::Amount { of_node: 10 })
                })
                // Hight of Fence posts
                .number_range(80..=100, |b| {
                    b.identifier(3)
                })
        })
        .node( |b| {
            b.identifier(10)
                .pos(|b| {
                    b.identifier(11)
                        .on_collapse(|wfc, user_data| {
                            (vec3(0.0, 0.0, 0.0), true)
                        })
                })
            });

    dbg!(&wfc_builder);

    let mut wfc = wfc_builder.build(&mut ());

    dbg!(&wfc);

    render_state.wfc_renderer.set_wfc(&wfc);

    let mut tree = CSGTree::new();
    wfc.show(&mut ());
    let data = tree.make_data();

    dbg!(&tree);

    render_state.csg_controller.set_data(&data);

    Ok(LogicState {
        camera,
        start_time: Instant::now(),
        wfc,
    })
}

#[no_mangle]
pub fn update(
    render_state: &mut RenderState,
    logic_state: &mut LogicState,
    engine: &mut Engine,
    frame_index: usize,
    delta_time: Duration,
) -> OctaResult<()> {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    let time = logic_state.start_time.elapsed();

    logic_state.camera.update(&engine.controls, delta_time);
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

    render_state.wfc_renderer.update(&engine.controls);

    Ok(())
}

#[no_mangle]
pub fn record_render_commands(
    render_state: &mut RenderState,
    _logic_state: &mut LogicState,
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

            render_state.wfc_renderer.gui_windows(ctx);
        },
    )?;

    command_buffer.end_rendering();

    Ok(())
}

#[no_mangle]
pub fn on_window_event(
    render_state: &mut RenderState,
    _logic_state: &mut LogicState,
    engine: &mut Engine,
    event: &WindowEvent,
) -> OctaResult<()> {
    render_state.gui.handle_event(&engine.window, event);

    Ok(())
}

#[no_mangle]
pub fn on_recreate_swapchain(
    render_state: &mut RenderState,
    _logic_state: &mut LogicState,
    engine: &mut Engine,
) -> OctaResult<()> {
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
