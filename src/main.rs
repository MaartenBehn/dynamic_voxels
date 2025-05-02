#![forbid(unused_must_use)]

use octa_force::binding::r#trait::BindingTrait;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::engine::{Engine, EngineConfig, EngineFeatureValue};
use octa_force::glam::uvec2;
use octa_force::hot_reloading::HotReloadConfig;
use octa_force::log::{error, trace};
use octa_force::OctaResult;
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
        gl_ext_scalar_block_layout: EngineFeatureValue::Needed,

        required_device_features: vec![
            "storagePushConstant8".to_string(),
            "bufferDeviceAddress".to_string(),
            "shaderInt8".to_string(),
            "variablePointersStorageBuffer".to_string(),
            "variablePointers".to_string(),
            "shaderInt64".to_string(),
        ],

        hot_reload_config: None, /*Some(HotReloadConfig {
            lib_dir: "target/debug".to_string(),
            lib_name: "reload".to_string(),
        }),*/

        ..Default::default()
    });
}

#[derive(Debug)]
pub struct App {}

impl BindingTrait for App {
    type RenderState = RenderState;
    type LogicState = LogicState;

    fn new_logic_state() -> OctaResult<Self::LogicState> {
        new_logic_state()
    }

    fn new_render_state(logic_state: &mut LogicState, engine: &mut Engine) -> OctaResult<Self::RenderState> {
        new_render_state(logic_state, engine)
    }

    fn update(
        logic_state: &mut LogicState,
        render_state: &mut RenderState,
        engine: &mut Engine,
        image_index: usize,
        delta_time: Duration,
    ) -> OctaResult<()> {
        update(logic_state, render_state, engine, image_index, delta_time)
    }

    fn record_render_commands(
        logic_state: &mut LogicState,
        render_state: &mut RenderState,
        engine: &mut Engine,
        image_index: usize,
    ) -> OctaResult<()> {
        record_render_commands(logic_state, render_state,  engine, image_index)
    }



    fn on_window_event(
        logic_state: &mut LogicState,
        render_state: &mut RenderState,
        engine: &mut Engine,
        event: &WindowEvent,
    ) -> OctaResult<()> {
        on_window_event(logic_state, render_state,  engine, event)
    }

    fn on_recreate_swapchain(
        logic_state: &mut LogicState,
        render_state: &mut RenderState,
        engine: &mut Engine,
    ) -> OctaResult<()> {
        on_recreate_swapchain(logic_state, render_state, engine)
    }
}
