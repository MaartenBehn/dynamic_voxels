#![forbid(unused_must_use)]

use octa_force::binding::r#trait::BindingTrait;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::glam::uvec2;
use octa_force::hot_reloading::HotReloadConfig;
use octa_force::log::{error, trace};
use octa_force::{Engine, EngineConfig, EngineFeatureValue, OctaResult};
use reload::{
    new_logic_state, new_render_state, on_recreate_swapchain, on_window_event,
    record_render_commands, update, LogicState, RenderState, USE_PROFILE,
};
use std::env;
use std::time::Duration;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const APP_NAME: &str = "Dynamic Voxels";

fn main() {
    octa_force::run::<App>(EngineConfig {
        name: APP_NAME.to_string(),
        start_size: uvec2(WIDTH, HEIGHT),
        ray_tracing: EngineFeatureValue::NotUsed,
        compute_rendering: EngineFeatureValue::Needed,
        validation_layers: EngineFeatureValue::Needed,
        shader_debug_printing: EngineFeatureValue::Needed,
        shader_debug_clock: if USE_PROFILE {
            EngineFeatureValue::Needed
        } else {
            EngineFeatureValue::NotUsed
        },
        GL_EXT_scalar_block_layout: EngineFeatureValue::Needed,
        hot_reload_config: Some(HotReloadConfig {
            lib_dir: "target/debug".to_string(),
            lib_name: "reload".to_string(),
        }),
    });
}
pub struct App {}

impl BindingTrait for App {
    type RenderState = RenderState;
    type LogicState = LogicState;

    fn new_render_state(engine: &mut Engine) -> OctaResult<Self::RenderState> {
        new_render_state(engine)
    }

    fn new_logic_state(
        render_state: &mut RenderState,
        engine: &mut Engine,
    ) -> OctaResult<Self::LogicState> {
        new_logic_state(render_state, engine)
    }

    fn update(
        render_state: &mut RenderState,
        logic_state: &mut LogicState,
        engine: &mut Engine,
        image_index: usize,
        delta_time: Duration,
    ) -> OctaResult<()> {
        update(render_state, logic_state, engine, image_index, delta_time)
    }

    fn record_render_commands(
        render_state: &mut RenderState,
        logic_state: &mut LogicState,
        engine: &mut Engine,
        image_index: usize,
    ) -> OctaResult<()> {
        record_render_commands(render_state, logic_state, engine, image_index)
    }

    fn on_window_event(
        render_state: &mut RenderState,
        logic_state: &mut LogicState,
        engine: &mut Engine,
        event: &WindowEvent,
    ) -> OctaResult<()> {
        on_window_event(render_state, logic_state, engine, event)
    }

    fn on_recreate_swapchain(
        render_state: &mut RenderState,
        logic_state: &mut LogicState,
        engine: &mut Engine,
    ) -> OctaResult<()> {
        on_recreate_swapchain(render_state, logic_state, engine)
    }
}
