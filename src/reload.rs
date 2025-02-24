mod aabb;
mod buddy_controller;
mod csg_tree;
mod color;
mod material;
mod profiler;
mod util;
mod model_synthesis;

use crate::csg_tree::controller::CSGController;
use crate::csg_tree::tree::{CSGTree, VOXEL_SIZE};
use crate::color::ColorController;
use crate::material::controller::MaterialController;
use crate::material::voxels::VoxelField;
use crate::profiler::ShaderProfiler;
use csg_tree::renderer::Renderer;
use csg_tree::tree::{CSGNode, CSGNodeData, MATERIAL_NONE};
use egui_graphs::Node;
use glsl_compiler::glsl;
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, Mat4, Quat, Vec3};
use octa_force::gui::Gui;
use octa_force::log::Log;
use octa_force::logger::setup_logger;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use octa_force::vulkan::Fence;
use octa_force::{log, Engine, OctaResult};
use model_synthesis::func_data::FuncData;
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use model_synthesis::builder::{NumberRangeDefinesType, WFCBuilder};
use model_synthesis::renderer::renderer::WFCRenderer;

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
            glsl! {type = Compute, profile, file = "shaders/trace_ray.glsl"}
        } else {
            glsl! {type = Compute, file = "shaders/trace_ray.glsl"}
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

#[derive(Debug, Clone)]
pub struct FenceData {
    pub center_pos: Vec3,
    pub radius: f32,
    pub post_distance: f32,
}

#[derive(Debug, Clone)]
pub struct FenceCSG {
    pub csg: CSGTree,
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


    let wfc_builder: WFCBuilder<FenceData, FenceCSG> = WFCBuilder::new()
        .node(|b| {
            b.identifier(0).name("Fence".to_owned())
                .user_data(FenceData {
                    center_pos: vec3(1.0, 1.0, 0.0),
                    radius: 10.0,
                    post_distance: 5.0,
                })
                // Number of fence posts
                .number_range(1, 5..=10, |b| {
                    b.defines(NumberRangeDefinesType::Amount { of_node: 10 })
                })
                // Hight of Fence posts
                .number_range(3, 80..=100, |b| {
                    b
                })
        })
        .node( |b| { // Fence Post
            b.identifier(10).name("Fence Post".to_owned())
                .pos(11, 
                10,
                |b| {
                    b
                })
                .build(|mut d| {
                    let pos = d.get_current_node_pos_attribute_with_identifier(11).unwrap().value;

                    let csg = &mut d.get_build_data_mut().csg;

                    let csg_node = CSGNode::new(CSGNodeData::Box(
                        Mat4::from_translation(pos),
                        MATERIAL_NONE,
                    ));

                    if csg.nodes.is_empty() {
                        csg.nodes.push(csg_node);
                        return;
                    }

                    let mut tree = CSGTree::from_node(csg_node);
                    csg.append_tree_with_union(tree);

                })
            });

    dbg!(&wfc_builder);

    let mut fence_csg = FenceCSG {
        csg: CSGTree::default()
    };

    wfc_builder.collapse(&mut fence_csg);

    render_state.csg_controller.set_data(&fence_csg.csg.make_data());
 

    Ok(LogicState {
        camera,
        start_time: Instant::now(),
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


    let mut tree = CSGTree::new_example_tree(time.as_secs_f32());
    render_state.csg_controller.set_data(&tree.make_data());


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
