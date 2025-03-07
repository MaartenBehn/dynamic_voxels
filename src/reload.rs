mod aabb;
mod buddy_controller;
mod csg_tree;
mod color;
mod profiler;
mod util;
mod model_synthesis;
mod voxel;

use crate::csg_tree::controller::CSGController;
use crate::csg_tree::tree::{CSGTree, VOXEL_SIZE};
use crate::color::ColorController;
use crate::profiler::ShaderProfiler;
use csg_tree::renderer::Renderer;
use csg_tree::tree::{CSGNode, CSGNodeData, MATERIAL_NONE};
use egui_graphs::Node;
use glsl_compiler::glsl;
use kiddo::SquaredEuclidean;
use model_synthesis::collapse::CollapseOperation;
use octa_force::camera::Camera;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::{vec3, Mat4, Quat, Vec3};
use octa_force::gui::Gui;
use octa_force::log::{debug, error, info, Log};
use octa_force::logger::setup_logger;
use octa_force::puffin_egui::puffin;
use octa_force::vulkan::ash::vk::AttachmentLoadOp;
use octa_force::vulkan::Fence;
use octa_force::{log, Engine, OctaResult};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use std::usize;
use model_synthesis::builder::{NumberRangeDefines, WFCBuilder, IT};
use model_synthesis::renderer::renderer::WFCRenderer;

pub const USE_PROFILE: bool = false;

pub struct RenderState {
    pub gui: Gui,
    pub csg_controller: CSGController,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Identifier {
    Fence,
    FencePost,
    PlankSetting,
    FencePlank,

    PostNumber,
    PostHeight,
    PostDistance,
    PostPos,
    PlankNumber,
    PlankDistance,
}
impl IT for Identifier {}


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
        &color_controller,
        &profiler,
        shader_bin,
    )?;

    // let mut tree = CSGTree::new_example_tree_2(1.0);
    // csg_controller.set_data(&tree.make_data());


    
    let wfc_renderer = WFCRenderer::new();

    let wfc_builder: WFCBuilder<Identifier> = WFCBuilder::new()
        .node(Identifier::Fence, |b| {
            b
                .number_range(Identifier::PostHeight, 3..=8, NumberRangeDefines::None, false)
                .number_range(Identifier::PostDistance, 2..=5, NumberRangeDefines::None, false)
                .number_range(Identifier::PostNumber, 5..=10, NumberRangeDefines::Amount { of_node: Identifier::FencePost }, false)
                .child(Identifier::PlankSetting)
        })
        .node(Identifier::FencePost, |b| {
            b.pos(Identifier::PostPos,false)
            .use_build_hook()
        })
        .node(Identifier::PlankSetting, |b| {
            b.number_range(Identifier::PlankNumber, 1..=4, NumberRangeDefines::Amount { of_node: Identifier::FencePlank }, false)
                .number_range(Identifier::PlankDistance, 1..=3, NumberRangeDefines::None, false)
        })
        .node(Identifier::FencePlank, |b| {
            b.use_build_hook()
        });
    
    let mut pos = vec3(1.0, 1.0, 1.0);
    let mut csg = CSGTree::default();

    let mut collapser = wfc_builder.get_collaper();
    while let Some((operation, collapser)) = collapser.next() {
        match operation {
            CollapseOperation::CollapsePos{ index  } => {
                let dist = collapser.get_number_with_identifier(Identifier::PostDistance);

                let pos_attribute = collapser.pos_attributes.get_mut(index).expect("Collapse: Pos Attribute not found!");
                pos_attribute.value = pos;

                pos += Vec3::X * dist as f32;
            },
            CollapseOperation::BuildNode{ index, identifier, .. } => {
                if identifier != Identifier::FencePost {
                    error!("Build hook on wrong type");
                    continue;
                }

                let node = collapser.nodes
                    .get(index)
                    .expect("Build: Node not found!");

                let pos_attribute = collapser.pos_attributes
                    .get(node.pos_attributes[0])
                    .expect("Build: Pos Attribute not found!");

                let height = collapser.get_number_with_identifier(Identifier::PostHeight);


                let pos = pos_attribute.value + Vec3::Z * (height as f32) * 0.5;

                let csg_node = CSGNode::new(CSGNodeData::Box(
                    Mat4::from_scale_rotation_translation(
                        vec3(0.5, 0.5, height as f32) * VOXEL_SIZE, 
                        Quat::IDENTITY, 
                        pos * VOXEL_SIZE
                    ),
                    1,
                ));
                debug!("{:?}: {}", identifier, pos);

                if csg.nodes.is_empty() {
                    csg.nodes.push(csg_node);
                    continue;
                }

                let mut tree = CSGTree::from_node(csg_node);
                csg.append_tree_with_union(tree);
                
            },
            CollapseOperation::None => {},
        } 
    }

    csg_controller.set_data(&csg.make_data());
    

    Ok(RenderState {
        gui,
        csg_controller,
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
