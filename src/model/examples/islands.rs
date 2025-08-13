
use std::sync::Arc;

use nalgebra::IsDynamic;
use octa_force::{camera::Camera, glam::{vec3, EulerRot, Mat4, Quat, Vec2, Vec3, Vec3A, Vec3Swizzles}, log::{debug, error, info}, vulkan::{AllocContext, Context, Swapchain}, OctaResult};
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{csg::{csg_tree::tree::{CSGNode, CSGTree}, csg_tree_2d::tree::CSGTree2D, fast_query_csg_tree::tree::FastQueryCSGTree}, model::generation::{builder::{BuilderAmmount, BuilderValue, ModelSynthesisBuilder}, collapse::{CollapseOperation, Collapser}, pos_set::{PositionSet, PositionSetRule}, template::TemplateTree, traits::{ModelGenerationTypes, BU, IT}}, scene::{dag64::{SceneAddDAGObject, SceneDAGObject}, dag_store::SceneDAGKey, renderer::SceneRenderer, worker::SceneWorkerSend, Scene, SceneObjectData, SceneObjectKey}, util::aabb3d::AABB, volume::{magica_voxel::MagicaVoxelModel, VolumeBoundsI, VolumeQureyPosValid}, voxel::{dag64::{parallel::ParallelVoxelDAG64, DAG64EntryKey, VoxelDAG64}, grid::{shared::SharedVoxelGrid, VoxelGrid}, palette::shared::SharedPalette}, METERS_PER_SHADER_UNIT};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Identifier {
    #[default]
    None,
    MinIslandDistance,
    IslandRadius,
    BeachWidth,
    IslandPositions,
    Island,
    TreePositions,
    TreeBuild,
    Rivers,
    RiverNode,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct IslandGenerationTypes {}
impl IT for Identifier {}
impl BU for Island {}
impl ModelGenerationTypes for IslandGenerationTypes {
    type Identifier = Identifier;
    type UndoData = Island;
    type Volume = CSGTree<()>;
    type Volume2D = CSGTree2D<()>;
}

#[derive(Clone, Debug)]
pub struct Islands {
    pub template: TemplateTree<IslandGenerationTypes>,
    pub collapser: Collapser<IslandGenerationTypes>,

    pub dag: ParallelVoxelDAG64,
    pub scene_dag_key: SceneDAGKey,
    pub last_pos: Vec3,
    pub tree_grid: SharedVoxelGrid,
}

#[derive(Clone, Debug, Default)]
pub struct Island {
    pub csg: CSGTree<u8>,
    pub scene_key: SceneObjectKey,
    pub dag_key: DAG64EntryKey,
}

pub struct UpdateData {
    pub pos: Vec3,
}

impl Islands {
    pub async fn new(palette: &mut SharedPalette, scene: &SceneWorkerSend) -> OctaResult<Self> {

        let mut dag = VoxelDAG64::new(1000000, 1000000).parallel();
        dag.print_memory_info();
        let scene_dag_key = scene.add_dag(dag.clone()).result_async().await;
        
        let tree_model = MagicaVoxelModel::new("./assets/Tree1small.vox")?;
        let tree_grid: SharedVoxelGrid = tree_model.into_grid(palette)?.into();
         
        let island_volume = CSGTree::default();

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
                    200.0 )
                ))
            })
            .build(Identifier::Island, |b| {b
                .ammount(BuilderAmmount::DefinedBy(Identifier::IslandPositions))
                .depends(Identifier::IslandPositions)
            })
            .position_set(Identifier::TreePositions, |b| {b
                .ammount(BuilderAmmount::OnePer(Identifier::Island))
                .value(BuilderValue::Hook)
                .depends(Identifier::IslandPositions)
            })
            
            .build(Identifier::TreeBuild, |b|{b
                .ammount(BuilderAmmount::DefinedBy(Identifier::TreePositions))
                .depends(Identifier::TreePositions)
                .depends(Identifier::Island)
            })

            .position_set(Identifier::Rivers, |b|{b
                .ammount(BuilderAmmount::OnePer(Identifier::Island))
                .value(BuilderValue::Const(PositionSet::new_path(
                5.0,
                    vec3(0.5, 0.5, 0.0),
                    vec3(-100.0, -100.0, 8.0),
                    vec3(100.0, 100.0, 8.0),
                )))
            })

            .build(Identifier::RiverNode, |b| {b
                .ammount(BuilderAmmount::DefinedBy(Identifier::Rivers))
                .depends(Identifier::Rivers)
                .depends(Identifier::Island)
            });

        let template = wfc_builder.build_template();

        let collapser = template.get_collaper();

        Ok(Self {
            template,
            collapser,
            dag,
            scene_dag_key,
            last_pos: Vec3::ZERO,
            tree_grid,
        })
    }

    pub fn update(&mut self, update_data: UpdateData) -> OctaResult<()> {
        let mut new_pos = update_data.pos;
        new_pos.z = 0.0;

        if new_pos == self.last_pos {
            return Ok(());
        }
        self.last_pos = new_pos;

        let island_volume = CSGTree::new_sphere(new_pos, 200.0); 

        self.template.get_node_position_set(Identifier::IslandPositions).set_volume(island_volume.clone());
        let pos_set = self.collapser.get_position_set_by_identifier_mut(Identifier::IslandPositions); 
        pos_set.set_volume(island_volume);
        self.collapser.re_collapse_all_nodes_with_identifier(Identifier::IslandPositions);

        Ok(())
    }

    pub async fn tick(&mut self, scene: &SceneWorkerSend, ticks: usize) -> OctaResult<bool> {
        let mut ticked = false;
        let mut i = 0;

        while let Some((hook, collapser)) 
            = self.collapser.next(&self.template) {
            ticked = true;

            match hook {
                CollapseOperation::NumberRangeHook { index, identifier } => {
                    info!("Number Range Hook");
                },
                CollapseOperation::PosSetHook { index, identifier } => {
                    info!("Pos Set Hook");

                    match identifier {
                        Identifier::TreePositions => {

                            //let mut pos = collapser.get_dependend_pos(index, Identifier::IslandPositions, Identifier::IslandBuild);
                            let tree_volume = CSGTree2D::new_circle(Vec2::ZERO, 200.0); 

                            collapser.set_position_set_value(index, PositionSet::new_grid_on_plane(
                                tree_volume,
                                100.0, 
                                0.0)
                            );    
                        },
                        _ => unreachable!()
                    }
                },
                CollapseOperation::RestrictHook { 
                    index, 
                    identifier, 
                    restricts_index, 
                    restricts_identifier } => {

                },
                CollapseOperation::BuildHook { index, identifier } => {

                    match identifier {
                        Identifier::Island => {

                            let pos = collapser.get_parent_pos(index);
                            info!("Island Pos: {pos}");

                            let csg = CSGTree::new_disk(Vec3::ZERO, 300.0, 10.0);
                            let active_key = self.dag.add_aabb_query_volume(&csg)?;

                            let scene_object_key = scene.add_dag_object(
                                Mat4::from_scale_rotation_translation(
                                    Vec3::ONE,
                                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                                    pos.to_owned() / METERS_PER_SHADER_UNIT as f32
                                ), 
                                self.scene_dag_key,
                                self.dag.get_entry(active_key),
                            ).result_async().await;

                            let island = Island {
                                csg,
                                scene_key: scene_object_key,
                                dag_key: active_key,
                            };

                            collapser.set_undo_data(index, island);

                        },
                        Identifier::TreeBuild => {
                            let pos = collapser.get_parent_pos(index);
                            let island = collapser.get_dependend_undo_data_mut(index, Identifier::Island);
                            let mut tree_grid = self.tree_grid.clone();
                            tree_grid.offset += pos.as_ivec3();

                            island.csg.append_node_with_union(CSGNode::new_shared_grid(tree_grid));
                            island.csg.calculate_bounds();

                            let active_key = self.dag.update_pos_query_volume(&island.csg, island.dag_key)?;
                            island.dag_key = active_key;
                            scene.set_dag_entry(island.scene_key, self.dag.get_entry(active_key));

                            info!("Tree Pos: {pos}");
                        },
                        Identifier::RiverNode => {
                            let pos = collapser.get_parent_pos(index);
                            let island = collapser.get_dependend_undo_data_mut(index, Identifier::Island);

                            island.csg.append_node_with_union(CSGNode::new_sphere(pos, 10.0));
                            island.csg.calculate_bounds();

                            let active_key = self.dag.update_pos_query_volume(&island.csg, island.dag_key)?;
                            island.dag_key = active_key;
                            scene.set_dag_entry(island.scene_key, self.dag.get_entry(active_key));

                            info!("River Pos: {pos}");
                        }
                        _ => unreachable!()
                    }

                },
                CollapseOperation::Undo { identifier , undo_data} => {
                    info!("Undo {:?}", identifier);

                    match identifier {
                        Identifier::Island => {
                            scene.remove_object(undo_data.scene_key);
                        },
                        _ => {}
                    }
                },
                CollapseOperation::None => {},
            } 

            i += 1;
            if i > ticks {
                break;
            }
        }

        Ok(ticked)
    }
}

