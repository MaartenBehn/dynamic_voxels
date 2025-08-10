extern crate reload as dynamic_voxels;

use bvh::bvh::Bvh;
use dynamic_voxels::csg::fast_query_csg_tree::tree::FastQueryCSGTree;
use dynamic_voxels::multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator;
use dynamic_voxels::voxel::dag64::VoxelDAG64;
use dynamic_voxels::voxel::grid::VoxelGrid;
use dynamic_voxels::model::examples::islands::Islands;
use octa_force::binding::r#trait::BindingTrait;
use octa_force::egui_winit::winit::event::WindowEvent;
use octa_force::engine::{Engine, EngineConfig, EngineFeatureValue};
use octa_force::glam::{uvec2, UVec3, Vec3};
use octa_force::hot_reloading::HotReloadConfig;
use octa_force::log::{self, error, info, trace};
use octa_force::simplelog::{self, SimpleLogger};
use octa_force::OctaResult;
use reload::{
    new_logic_state, new_render_state, on_recreate_swapchain, on_window_event, record_render_commands, update, LogicState, RenderState, NUM_FRAMES_IN_FLIGHT, USE_PROFILE
};
use std::{env, usize};
use std::time::Duration;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const APP_NAME: &str = "Dynamic Voxels";

fn main() {
    #[cfg(feature = "profile_islands")]
    {
        /*
        SimpleLogger::init(log::LevelFilter::Debug, simplelog::Config::default());
        let mut state = Islands::new(true);
        state.run();

        let collapser = state.collapser.unwrap(); 
        info!("Node capacity: {}", collapser.nodes.capacity());
        info!("Pending collapse operations capacity: {}", collapser.pending_collapse_opperations.capacity());

        */
        return;
    }

    #[cfg(feature = "profile_dag")]
    { 
        /*
        let buffer_size = 2_usize.pow(30);
        let mut allocator = BuddyBufferAllocator::new(buffer_size, 32);

        let USE_CSG = true;
        let tree64 = if USE_CSG {
            let csg: FastQueryCSGTree<u8> = VecCSGTree::new_sphere(Vec3::ZERO, 100.0).into();
            VoxelDAG64::from_aabb_query(&csg, &mut allocator).unwrap()
        } else {
            let mut grid = VoxelGrid::new(UVec3::ONE * 4_u32.pow(5)); 
            grid.set_example_sphere();
            grid.set_corners();

            VoxelDAG64::from_pos_query(&grid, &mut allocator).unwrap()
        }; 
        dbg!(tree64.root_index);
        
        */
        return;
    }

    octa_force::run::<App>(EngineConfig {
        name: APP_NAME.to_string(),
        start_size: uvec2(WIDTH, HEIGHT),
        num_frames_in_flight: NUM_FRAMES_IN_FLIGHT,

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

        required_extensions: vec![
            "VK_KHR_shader_clock".to_string(),
        ],

        required_device_features: vec![
            "storagePushConstant8".to_string(),
            "bufferDeviceAddress".to_string(),
            "shaderInt8".to_string(),
            "shaderInt64".to_string(),
            "descriptorBindingStorageImageUpdateAfterBind".to_string(),
            "descriptorBindingSampledImageUpdateAfterBind".to_string(),
            "descriptorBindingUpdateUnusedWhilePending".to_string(),
            "descriptorBindingPartiallyBound".to_string(),
            "shaderSubgroupClock".to_string(),
            "runtimeDescriptorArray".to_string(),
            "shaderBufferInt64Atomics".to_string(),
            "shaderInt16".to_string(),

            // For DescriptorHeap
            "variablePointersStorageBuffer".to_string(),
            "variablePointers".to_string(),
            //"shaderFloat16".to_string(),
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
        delta_time: Duration,
    ) -> OctaResult<()> {
        update(logic_state, render_state, engine, delta_time)
    }

    fn record_render_commands(
        logic_state: &mut LogicState,
        render_state: &mut RenderState,
        engine: &mut Engine,
    ) -> OctaResult<()> {
        record_render_commands(logic_state, render_state,  engine)
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
