
use std::{sync::Arc};

use octa_force::{camera::Camera, glam::{vec3, EulerRot, Mat4, Quat, Vec3, Vec3A, Vec3Swizzles}, log::{error, info}, vulkan::{Context, Swapchain}, OctaResult};
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{csg::{csg_tree_2d::tree::CSGTree2D, fast_query_csg_tree::tree::FastQueryCSGTree, slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGTree, SlotMapCSGTreeKey}, vec_csg_tree::tree::VecCSGTree}, model::generation::{builder::{BuilderAmmount, BuilderValue, ModelSynthesisBuilder}, collapse::{CollapseOperation, Collapser}, pos_set::{PositionSet, PositionSetRule}, template::TemplateTree, traits::{ModelGenerationTypes, BU, IT}}, scene::{dag64::DAG64SceneObject, renderer::SceneRenderer, Scene, SceneObjectData, SceneObjectKey}, volume::VolumeQureyPosValid, voxel::dag64::{DAG64EntryKey, VoxelDAG64}, METERS_PER_SHADER_UNIT};

new_key_type! { pub struct IslandKey; }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Identifier {
    #[default]
    None,
    MinIslandDistance,
    IslandRadius,
    BeachWidth,
    IslandPositions,
    IslandBuild,
    TreePositions,
    TreePosition,
}


#[derive(Clone, Copy, Debug, Default)]
pub struct IslandGenerationTypes {}
impl IT for Identifier {}
impl BU for IslandKey {}
impl ModelGenerationTypes for IslandGenerationTypes {
    type Identifier = Identifier;
    type UndoData = IslandKey;
    type Volume = FastQueryCSGTree<()>;
    type Volume2D = CSGTree2D<()>;
}

#[derive(Clone, Debug)]
pub struct IslandsState {
    pub template: TemplateTree<IslandGenerationTypes>,
    pub collapser: Collapser<IslandGenerationTypes>,

    islands: SlotMap<IslandKey, IslandState>,
    pub dag: Arc<Mutex<VoxelDAG64>>,
    pub last_pos: Vec3,
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
         
        let island_volume = FastQueryCSGTree::default();

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
            .position_set(Identifier::IslandPositions, |b| {b
                .ammount(BuilderAmmount::OneGlobal)
                .value(BuilderValue::Const(PositionSet::new_grid_in_volume(
                    island_volume,
                    if profile { 0.1 } else { 20.0 })
                ))
            })
            .build(Identifier::IslandBuild, |b| {b
                .ammount(BuilderAmmount::DefinedBy(Identifier::IslandPositions))
                .depends(Identifier::IslandPositions)
            })
            .position_set(Identifier::TreePositions, |b| {b
                .ammount(BuilderAmmount::OnePer(Identifier::IslandBuild))
                .value(BuilderValue::Hook)
                .depends(Identifier::IslandPositions)
            })
            
            .build(Identifier::TreePosition, |b|{b
                .ammount(BuilderAmmount::DefinedBy(Identifier::TreePositions))
                .depends(Identifier::TreePositions)
            });

        let template = wfc_builder.build_template();
        dbg!(&template);

        let collapser = template.get_collaper();

        Ok(Self {
            template,
            collapser,
            islands: Default::default(),
            dag: Arc::new(Mutex::new(dag)),
            last_pos: Vec3::ZERO,
        })
    }

    pub fn update(&mut self, camera: &Camera) -> OctaResult<()> {

        let mut new_pos = camera.get_position_in_meters();
        new_pos.z = 0.0;

        if new_pos == self.last_pos {
            return Ok(());
        }
        self.last_pos = new_pos;

        let island_volume = VecCSGTree::new_sphere(new_pos, 40.0); 
        let island_volume = FastQueryCSGTree::from(island_volume);

        self.template.get_node_position_set(Identifier::IslandPositions)?.set_volume(island_volume.clone())?;
        if let Ok(pos_set) = self.collapser.get_position_set_by_identifier_mut(Identifier::IslandPositions) {
            pos_set.set_volume(island_volume)?;
            self.collapser.re_collapse_all_nodes_with_identifier(Identifier::IslandPositions);
        }

        Ok(())
    }

    pub fn tick(&mut self, scene: &mut Scene, context: &Context) -> OctaResult<bool> {
        let mut ticked = false;

        if let Some((hook, collapser)) 
            = self.collapser.next(&self.template)? {

            ticked = true;

            match hook {
                CollapseOperation::NumberRangeHook { index, identifier } => {
                    info!("Number Range Hook");
                },
                CollapseOperation::PosSetHook { index, identifier } => {
                    info!("Pos Set Hook");

                    match identifier {
                        Identifier::TreePositions => {

                            let mut pos = collapser.get_dependend_pos(index, Identifier::IslandPositions, Identifier::IslandBuild);
                            let tree_volume = CSGTree2D::new_circle(pos.xy(), 10.0); 

                            collapser.set_position_set_value(index, PositionSet::new_grid_on_plane(
                                tree_volume,
                                5.0, 
                                pos.z)
                            );    
                        },
                        _ => unreachable!()
                    }
                },
                CollapseOperation::BuildHook { index, identifier } => {

                    match identifier {
                        Identifier::IslandBuild => {
                            let pos = collapser.get_parent_pos(index);

                            let csg = SlotMapCSGTree::new_sphere(Vec3::ZERO, 10.0);
                            let active_key = self.dag.lock().add_aabb_query_volume(&csg)?;

                            let scene_object_key = scene.add_dag64(
                                context,
                                Mat4::from_scale_rotation_translation(
                                    Vec3::ONE,
                                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                                    pos.to_owned() / METERS_PER_SHADER_UNIT as f32
                                ), 
                                active_key,
                                self.dag.clone(),
                            )?;

                            let island_state = IslandState {
                                csg,
                                active_key,
                                scene_object_key,
                            };

                            let island_key = self.islands.insert(island_state);
                            collapser.set_undo_data(index, island_key)?;

                        },
                        Identifier::TreePosition => {
                            let pos = collapser.get_parent_pos(index);

                            info!("Tree Pos: {pos}");
                        },
                        _ => unreachable!()
                    }

                },
                CollapseOperation::Undo { identifier , undo_data} => {
                    info!("Undo {:?}", identifier);

                    let island_state = self.islands.remove(undo_data).unwrap();
                    scene.remove_object(island_state.scene_object_key)?;
                },
                CollapseOperation::None => {},
            } 
        }

        if !ticked {
            //info!("Idle");
        }

        Ok(ticked)
    }
}

