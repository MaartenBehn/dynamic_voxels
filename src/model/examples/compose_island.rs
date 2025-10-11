use std::{marker::PhantomData, time::Duration};

use egui_snarl::{NodeId, OutPinId};
use octa_force::{egui, glam::{EulerRot, IVec2, IVec3, Mat4, Quat, Vec3}, OctaResult};
use slotmap::Key;
use smallvec::SmallVec;

use crate::{model::{collapse::collapser::{CollapseNode, NodeDataType}, composer::{build::{CollapseValueTrait, ComposeTypeTrait, GetTemplateValueArgs, OnCollapseArgs, OnDeleteArgs, TemplateValueTrait, BS}, nodes::{ComposeNode, ComposeNodeGroupe, ComposeNodeInput, ComposeNodeType}, template::TemplateIndex, ModelComposer}, data_types::{data_type::ComposeDataType, position::PositionTemplate, volume::VolumeTemplate}}, scene::{dag_store::SceneDAGKey, worker::SceneWorkerSend, SceneObjectKey}, util::{number::Nu, vector::Ve}, volume::VolumeBounds, voxel::{dag64::{parallel::ParallelVoxelDAG64, DAG64EntryKey, VoxelDAG64}, palette::palette::MATERIAL_ID_BASE}, METERS_PER_SHADER_UNIT};

// Compose Type
#[derive(Debug)]
pub struct ComposeIsland {
    pub composer: ModelComposer<IVec2, IVec3, i32, ComposeIslandState>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComposeType {
    Object
}
impl ComposeTypeTrait for ComposeType {}


// Template Value
#[derive(Debug, Clone)]
pub enum TemplateValue<V3: Ve<T, 3>, V2: Ve<T, 2>, T: Nu> {
    Object(ObjectTemplate<V3, V2, T>)
}

#[derive(Debug, Clone)]
pub struct ObjectTemplate<V3: Ve<T, 3>, V2: Ve<T, 2>, T: Nu> {
    pos: PositionTemplate<V3, V2, V3, T, 3>,
    volume: VolumeTemplate<V3, V2, V3, T, 3>,
}

impl<V3: Ve<T, 3>, V2: Ve<T, 2>, T: Nu> TemplateValueTrait for TemplateValue<V3, V2, T> {
    fn get_dependend_template_nodes(&self) -> SmallVec<[TemplateIndex; 4]> {
        match self {
            TemplateValue::Object(object_template) => {
                object_template.volume.get_dependend_template_nodes().collect()
            },
        }
    }
}

// Collapse Value

#[derive(Debug, Clone)]
pub enum CollapseValue {
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
    type TemplateValue = TemplateValue<V3, V2, T>;
    type CollapseValue = CollapseValue; 

    fn compose_nodes() -> Vec<ComposeNode<Self::ComposeType>> {
        vec![
            ComposeNode { 
                t: ComposeNodeType::Build(ComposeType::Object), 
                id: NodeId(usize::MAX),
                group: ComposeNodeGroupe::Build, 
                inputs: vec![
                    ComposeNodeInput { 
                        name: "ammount".to_string(), 
                        data_type: ComposeDataType::Ammount,
                        valid: false,
                    },
                    ComposeNodeInput { 
                        name: "volume".to_string(), 
                        data_type: ComposeDataType::Volume3D, 
                        valid: false,
                    },
                    ComposeNodeInput { 
                        name: "pos".to_string(), 
                        data_type: ComposeDataType::Position3D(None), 
                        valid: false,
                    },
                ], 
                outputs: vec![], 
            },
        ]
    }

    fn is_template_node(t: &Self::ComposeType) -> bool { 
        true
    }

    fn get_template_value(args: GetTemplateValueArgs<V2, V3, T, Self>) -> Self::TemplateValue {
        match args.compose_type {
            ComposeType::Object => {
                let volume = args.composer.make_volume(
                    args.composer.get_input_remote_pin_by_type(args.composer_node, ComposeDataType::Volume3D),
                    args.building_template_index, args.template);

                let pos = args.composer.make_position(
                    args.composer_node,
                    args.composer.get_input_pin_index_by_type(args.composer_node, ComposeDataType::Position3D(None)),
                    args.building_template_index, args.template);
 
                TemplateValue::Object(ObjectTemplate { volume, pos }) 
            },
        }
    }

    async fn on_collapse<'a>(args: OnCollapseArgs<'a, V2, V3, T, Self>) -> Self::CollapseValue {
        delete_object(args.collapse_node, args.state);

        match args.template_value {
            TemplateValue::Object(object_template) => {
                
                let (mut volume, r_0) = object_template.volume.get_value(args.get_value_data, args.collapser, MATERIAL_ID_BASE);
                let (pos, r_1) = object_template.pos.get_value(args.get_value_data, args.collapser);

                volume.calculate_bounds();

                let dag_key = args.state.dag.add_pos_query_volume(&volume).expect("Could not add DAG Entry!");
                let scene_key = args.state.scene.add_dag_object(
                    Mat4::from_scale_rotation_translation(
                        Vec3::ONE,
                        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                        pos.to_vec3() / METERS_PER_SHADER_UNIT as f32
                    ), 
                    args.state.scene_dag_key,
                    args.state.dag.get_entry(dag_key),
                ).result_async().await;

                CollapseValue::Object(Object { scene_key, dag_key })
            },
        }
    }

    fn on_delete(args: OnDeleteArgs<V2, V3, T, Self>) {
        delete_object(args.collapse_node, args.state);
    }
}

fn delete_object<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu>(node: &'a CollapseNode<V2, V3, T, ComposeIslandState>, state: &'a mut ComposeIslandState) {
    match &node.data {
        NodeDataType::Build(t) => match t {
            CollapseValue::Object(object) => {
                state.scene.remove_object(object.scene_key);
            },
        },
        _ => {}
    }
}

impl ComposeIsland {
    pub fn new(scene: SceneWorkerSend) -> Self {
        let mut dag = VoxelDAG64::new(1000000, 1000000).parallel();
        let scene_dag_key = scene.add_dag(dag.clone()).result_blocking();
        let mut state =  ComposeIslandState {
            dag,
            scene,
            scene_dag_key,
        }; 

        Self {
            composer: ModelComposer::new(state),
        }
    }

    pub fn update(&mut self, time: Duration) -> OctaResult<()> {
        self.composer.update(time)
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        self.composer.render(ctx);
    }
}


