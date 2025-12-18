use std::{marker::PhantomData, sync::Arc, time::{Duration, Instant}};

use egui_snarl::{NodeId, OutPinId};
use octa_force::{camera::Camera, egui, glam::{EulerRot, IVec2, IVec3, Mat4, Quat, Vec3}, log::info, OctaResult};
use slotmap::Key;
use smallvec::SmallVec;

use crate::{METERS_PER_SHADER_UNIT, VOXELS_PER_SHADER_UNIT, csg::csg_tree::tree::CSGTree, model::{collapse::collapser::{CollapseNode, NodeDataType}, composer::{ModelComposer, build::{BS, CollapseValueTrait, ComposeTypeTrait, GetTemplateValueArgs, OnCollapseArgs, OnDeleteArgs, TemplateValueTrait}, nodes::{ComposeNode, ComposeNodeGroupe, ComposeNodeInput, ComposeNodeType}}, data_types::{data_type::ComposeDataType, position::{PositionTemplate, ValueIndexPosition}, volume::{ValueIndexVolume, VolumeTemplate}}, template::update::MakeTemplateData}, scene::{SceneObjectKey, dag_store::SceneDAGKey, worker::SceneWorkerSend}, util::{number::Nu, vector::Ve}, volume::VolumeBounds, voxel::{dag64::{DAG64EntryKey, VoxelDAG64, parallel::ParallelVoxelDAG64}, palette::{palette::MATERIAL_ID_BASE, shared::SharedPalette}}};

// Compose Type
#[derive(Debug)]
pub struct ComposeIsland {
    pub composer: ModelComposer<IVec2, IVec3, i32, ComposeIslandState>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComposeType {
    Object
}
impl ComposeTypeTrait for ComposeType {
}


// Template Value
#[derive(Debug, Clone)]
pub enum TemplateValue {
    Object(ObjectTemplate)
}

#[derive(Debug, Clone)]
pub struct ObjectTemplate {
    pos: ValueIndexPosition,
    volume: ValueIndexVolume,
}

impl TemplateValueTrait for TemplateValue {}

// Collapse Value
#[derive(Debug, Clone, Default)]
pub enum CollapseValue {
    #[default]
    None,
    Object(Object)
}

#[derive(Debug, Clone)]
pub struct Object {
    pub scene_key: SceneObjectKey,
    pub dag_key: DAG64EntryKey,
}

impl CollapseValueTrait for CollapseValue {}


#[derive(Debug, Clone)]
pub struct ComposeIslandState {
    pub dag: ParallelVoxelDAG64,
    pub scene: SceneWorkerSend,
    pub scene_dag_key: SceneDAGKey,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> BS<V2, V3, T> for ComposeIslandState {
    type ComposeType = ComposeType;
    type TemplateValue = TemplateValue;
    type CollapseValue = CollapseValue; 

    fn compose_nodes() -> Vec<ComposeNode<Self::ComposeType>> {
        vec![
            ComposeNode::new(ComposeNodeType::Build(ComposeType::Object), ComposeNodeGroupe::Build)
                .input(ComposeDataType::Volume3D, "volume")
                .input(ComposeDataType::Position3D(None), "pos")
                .input(ComposeDataType::Creates, "one per"),
        ]
    }

    fn is_template_node(t: &Self::ComposeType) -> bool { 
        true
    }

    fn get_template_value(mut args: GetTemplateValueArgs<V2, V3, T, Self>, data: &mut MakeTemplateData<V2, V3, T, Self>) -> Self::TemplateValue {
        match args.compose_type {
            ComposeType::Object => {
                let volume = args.graph.make_volume(args.composer_node, 0, data);
                let pos = args.graph.make_position(args.composer_node,1, data);
 
                TemplateValue::Object(ObjectTemplate { volume, pos }) 
            },
        }
    }

    async fn on_collapse<'a>(args: OnCollapseArgs<'a, V2, V3, T, Self>) -> Self::CollapseValue {

        match args.template_value {
            TemplateValue::Object(object_template) => {

                let (mut volume, r_0) = args.template.get_volume_value(object_template.volume)
                    .get_value::<V3, V2, V3, T, Self, u8, 3>(args.get_value_data, args.collapser, args.template);

                let (mut pos, r_1) = args.template.get_position3d_value(object_template.pos)
                    .get_value(args.get_value_data, args.collapser, args.template);

                let pos = pos[0];

                volume.calculate_bounds();

                let now = Instant::now();
                let dag_key = args.state.dag.add_aabb_query_volume(&volume).expect("Could not add DAG Entry!");
                let elapsed = now.elapsed();
                info!("Voxel DAG Build took: {:?}", elapsed);

                delete_object(args.collapse_value, args.state);
                let scene_key = args.state.scene.add_dag_object(
                    Mat4::from_scale_rotation_translation(
                        Vec3::ONE,
                        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                        pos.to_vec3() / VOXELS_PER_SHADER_UNIT as f32
                    ), 
                    args.state.scene_dag_key,
                    args.state.dag.get_entry(dag_key),
                ).result_async().await;

                CollapseValue::Object(Object { scene_key, dag_key })
            },
        }
    }

    fn on_delete(args: OnDeleteArgs<V2, V3, T, Self>) {
        delete_object(args.collapse_value, args.state);
    }
}

fn delete_object(
    collapse_value: &CollapseValue, 
    state: &mut ComposeIslandState
) {
    match collapse_value {
        CollapseValue::Object(object) => {
                state.scene.remove_object(object.scene_key);
            },
        CollapseValue::None => {},
    }
}

impl ComposeIsland {
    pub fn new(scene: SceneWorkerSend, camera: &Camera, palette: SharedPalette) -> Self {
        let mut dag = VoxelDAG64::new(1000000, 1000000).parallel();
        let scene_dag_key = scene.add_dag(dag.clone()).result_blocking();
        let mut state =  ComposeIslandState {
            dag,
            scene,
            scene_dag_key,
        }; 

        Self {
            composer: ModelComposer::new(state, camera, palette),
        }
    }

    pub fn update(&mut self, time: Duration, camera: &Camera) -> OctaResult<()> {
        self.composer.update(time, camera)
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        self.composer.render(ctx);
    }
}


