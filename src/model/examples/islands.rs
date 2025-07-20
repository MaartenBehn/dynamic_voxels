
use std::{sync::Arc};

use octa_force::{camera::Camera, glam::{vec3, EulerRot, Mat4, Quat, Vec3}, log::{error, info}, vulkan::{Context, Swapchain}, OctaResult};
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{csg::{fast_query_csg_tree::tree::FastQueryCSGTree, slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGTree, SlotMapCSGTreeKey}, vec_csg_tree::tree::VecCSGTree}, model::generation::{builder::{BuilderAmmount, BuilderValue, ModelSynthesisBuilder, IT}, collapse::{CollapseOperation, Collapser}, collapser_data::CollapserData, pos_set::{PositionSet, PositionSetRule}, template::TemplateTree}, scene::{dag64::DAG64SceneObject, renderer::SceneRenderer, Scene, SceneObjectData, SceneObjectKey}, volume::VolumeQureyPosValid, voxel::dag64::{DAG64EntryKey, VoxelDAG64}, METERS_PER_SHADER_UNIT};


#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Identifier {
    #[default]
    None,
    MinIslandDistance,
    IslandRadius,
    BeachWidth,
    IslandRoot,
    IslandPos,
    IslandBuild,
}
impl IT for Identifier {}

new_key_type! { pub struct IslandKey; }

#[derive(Clone, Debug)]
pub struct IslandsState {
    pub template: TemplateTree<Identifier, FastQueryCSGTree<()>>,
    pub collapser: Option<CollapserData<Identifier, SlotMapCSGTreeKey, FastQueryCSGTree<()>>>,

    pub islands: SlotMap<IslandKey, IslandState>,
    pub dag: Arc<Mutex<VoxelDAG64>>,
}

#[derive(Clone, Debug)]
struct IslandState {
    pub csg: SlotMapCSGTree<u8>,
    pub active_key: DAG64EntryKey,
    pub scene_object_key: SceneObjectKey,
}

impl IslandsState {
    pub fn new(profile: bool) -> OctaResult<Self> {

        let dag = VoxelDAG64::new(10000, 64);
         
        let island_volume = VecCSGTree::new_disk(Vec3::ZERO, 200.0, 0.1); 
        let island_volume = FastQueryCSGTree::from(island_volume);

        let mut wfc_builder = ModelSynthesisBuilder::new()
            .number_range(Identifier::MinIslandDistance, |b|{b
                .ammount(BuilderAmmount::OneGlobal)
                .value(BuilderValue::Const(3..=10))
            })

            .number_range(Identifier::IslandRadius, |b|{b
                .ammount(BuilderAmmount::OneGlobal)
                .value(BuilderValue::Const(1..=3))
            })

            .number_range(Identifier::BeachWidth, |b| {b
                .ammount(BuilderAmmount::OneGlobal)
                .value(BuilderValue::Const(0..=2))
            })

            .position_set(Identifier::IslandRoot, |b| {b
                .ammount(BuilderAmmount::OneGlobal)
                .value(BuilderValue::Const(PositionSet::new(
                    island_volume,
                    PositionSetRule::Grid { spacing: (if profile { 0.1 } else { 50.0 }) })))
            })

            .pos(Identifier::IslandPos, |b| {b
                .ammount(BuilderAmmount::DefinedBy(Identifier::IslandRoot))
            })

            .build(Identifier::IslandBuild, |b| {b
                .ammount(BuilderAmmount::OnePer(Identifier::IslandPos))
                .depends(Identifier::IslandPos)
            });

        let template = wfc_builder.build_template();

        let collapser = template.get_collaper().into_data();

        Ok(Self {
            template,
            collapser: Some(collapser),
            islands: Default::default(),
            dag: Arc::new(Mutex::new(dag)),
        })
    }

    pub fn tick(&mut self, scene: &mut Scene, context: &Context) -> OctaResult<bool> {
        let mut ticked = false;
                    
        
        let mut collapser = self.collapser.take().unwrap().into_collapser(&self.template);
        if let Some((hook, collapser)) = collapser.next()? {
            ticked = true;

            match hook {
                CollapseOperation::NumberRangeHook { index } => {
                    #[cfg(not(feature = "profile_islands"))]
                    info!("Number Range Hook")
                },
                CollapseOperation::PosSetHook { index } => {
                    #[cfg(not(feature = "profile_islands"))]
                    info!("Pos Set Hook")
                },
                CollapseOperation::PosHook { index } => {
                    #[cfg(not(feature = "profile_islands"))]
                    info!("Pos Hook")
                },
                CollapseOperation::BuildHook { index, identifier } => {  
                    let pos = collapser.get_dependend_pos(index, Identifier::IslandPos);

                    #[cfg(not(feature = "profile_islands"))]
                    info!("Build Hook: {pos}");

                    let csg = SlotMapCSGTree::new_sphere(Vec3::ZERO, 10.0);
                    let active_key = self.dag.lock().add_aabb_query_volume(&csg)?;

                    let scene_object_key = scene.add_dag64(
                        context,
                        Mat4::from_scale_rotation_translation(
                            Vec3::ONE,
                            Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                            pos / METERS_PER_SHADER_UNIT as f32
                        ), 
                        active_key,
                        self.dag.clone(),
                    )?;

                    let island_state = IslandState {
                        csg,
                        active_key,
                        scene_object_key,
                    };

                    self.islands.insert(island_state);
                },
                CollapseOperation::Undo { identifier , undo_data} => {
                    #[cfg(not(feature = "profile_islands"))]
                    info!("Undo {:?}", identifier);

                },
                CollapseOperation::None => {},
            } 
        }

        self.collapser = Some(collapser.into_data());

        Ok(ticked)
    }
}

