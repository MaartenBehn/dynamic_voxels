
use std::sync::Arc;

use octa_force::{anyhow::bail, camera::Camera, glam::{vec3, vec3a, EulerRot, Mat4, Quat, Vec2, Vec3, Vec3A, Vec3Swizzles}, log::{debug, error, info}, vulkan::{AllocContext, Context, Swapchain}, OctaResult};
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{csg::{sphere::CSGSphere, union::tree::CSGUnion}, model::{generation::{builder::{BuilderAmmount, BuilderValue, ModelSynthesisBuilder}, collapse::{CollapseOperation, Collapser}, pos_set::{PositionSet, PositionSetRule}, template::TemplateTree, traits::{Model, ModelGenerationTypes, BU, IT}}, worker::ModelChangeSender}, scene::{dag64::{SceneAddDAGObject, SceneDAGObject}, dag_store::SceneDAGKey, renderer::SceneRenderer, worker::SceneWorkerSend, Scene, SceneObjectData, SceneObjectKey}, util::{math_config::{Float2D, Float3D, Int3D}}, volume::{magica_voxel::MagicaVoxelModel, VolumeBounds, VolumeQureyPosValid}, voxel::{dag64::{parallel::ParallelVoxelDAG64, DAG64EntryKey, VoxelDAG64}, grid::{shared::SharedVoxelGrid, VoxelGrid}, palette::shared::SharedPalette}, METERS_PER_SHADER_UNIT};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Identifier {
    #[default]
    Root,
    IslandPositions,
    Island,
    TreePositions,
    Tree,
    Rivers,
    RiverNode,
    IslandDone
}

#[derive(Clone, Debug, Default)]
pub enum UndoData {
    #[default]
    None,
    Island(Island),
    IslandDone(IslandDone),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct IslandGenerationTypes {}
impl IT for Identifier {}
impl BU for UndoData {}
impl ModelGenerationTypes for IslandGenerationTypes {
    type Identifier = Identifier;
    type UndoData = UndoData;
    type Volume = CSGSphere<(), Float3D, 3>;
    type Volume2D = CSGSphere<(), Float2D, 2>;
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
    pub union: CSGUnion<u8, Int3D, 3>,
}

#[derive(Clone, Debug, Default)]
pub struct IslandDone {
    pub scene_key: SceneObjectKey,
    pub dag_key: DAG64EntryKey,
}

#[derive(Debug)]
pub struct IslandUpdateData {
    pub pos: Vec3,
}

impl Model for Islands {
    type GenerationTypes = IslandGenerationTypes;
    type UpdateData = IslandUpdateData;

    async fn new(palette: &mut SharedPalette, scene: &SceneWorkerSend, change: &ModelChangeSender<IslandGenerationTypes>) -> OctaResult<Self> {

        let mut dag = VoxelDAG64::new(1000000, 1000000).parallel();
        dag.print_memory_info();
        let scene_dag_key = scene.add_dag(dag.clone()).result_async().await;
        
        let tree_model = MagicaVoxelModel::new("./assets/Tree1small.vox")?;
        let tree_grid: SharedVoxelGrid = tree_model.into_grid(palette)?.into();
         
    let island_volume = CSGSphere::default();

        let mut wfc_builder = ModelSynthesisBuilder::new()
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
            
            .build(Identifier::Tree, |b|{b
                .ammount(BuilderAmmount::DefinedBy(Identifier::TreePositions))
                .depends(Identifier::TreePositions)
                .depends(Identifier::Island)
            })

            .position_set(Identifier::Rivers, |b|{b
                .ammount(BuilderAmmount::OnePer(Identifier::Island))
                .value(BuilderValue::Const(PositionSet::new_path(
                5.0,
                    vec3a(0.5, 0.5, 0.0),
                    vec3a(-100.0, -100.0, 8.0),
                    vec3a(100.0, 100.0, 8.0),
                )))
            })

            .build(Identifier::RiverNode, |b| {b
                .ammount(BuilderAmmount::DefinedBy(Identifier::Rivers))
                .depends(Identifier::Rivers)
                .depends(Identifier::Island)
            })

            .build(Identifier::IslandDone, |b| {b
                .ammount(BuilderAmmount::OnePer(Identifier::Island))
                .depends(Identifier::RiverNode)
                .depends(Identifier::IslandPositions)
            });

        let template = wfc_builder.build_template();
        change.send_template(template.clone());

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

    async fn update(&mut self, update_data: IslandUpdateData, change: &ModelChangeSender<IslandGenerationTypes>) -> OctaResult<()> {
        let mut new_pos = update_data.pos;
        new_pos.z = 0.0;

        if new_pos == self.last_pos {
            return Ok(());
        }
        self.last_pos = new_pos;

        let island_volume = CSGSphere::new_sphere(new_pos.into(), 400.0); 

        self.template.get_node_position_set(Identifier::IslandPositions)?.set_volume(island_volume.clone())?;

        if let Ok(pos_set) = self.collapser.get_position_set_by_identifier_mut(
            Identifier::IslandPositions) {

            pos_set.set_volume(island_volume)?;

            self.collapser.re_collapse_all_nodes_with_identifier(Identifier::IslandPositions);
        }

        Ok(())
    }

    async fn tick(&mut self, scene: &SceneWorkerSend, change: &ModelChangeSender<IslandGenerationTypes>) -> OctaResult<bool> {
        let mut ticked = false;

        while let Some((hook, collapser)) 
            = self.collapser.next(&self.template) {
            ticked = true;

            change.send_collapser(collapser.clone());

            match hook {
                CollapseOperation::NumberRangeHook { index, identifier } => {
                    info!("Number Range Hook");
                },
                CollapseOperation::PosSetHook { index, identifier } => {
                    info!("Pos Set Hook");

                    match identifier {
                        Identifier::TreePositions => {

                            //let mut pos = collapser.get_dependend_pos(index, Identifier::IslandPositions, Identifier::IslandBuild);
                            let tree_volume = CSGSphere::new_sphere(Vec2::ZERO, 300.0); 

                            collapser.set_position_set_value(index, PositionSet::new_grid_on_plane(
                                tree_volume,
                                50.0, 
                                0.0)
                            )?;    
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

                            let pos = collapser.get_parent_pos(index)?;
                            info!("Island Pos: {pos}");

                            let mut u = CSGUnion::new();
                            u.add_disk(Vec3A::ZERO, 300.0, 10.0);

                            collapser.set_undo_data(index, UndoData::Island(Island {
                                union: u,
                            }));
                        },
                        Identifier::Tree => {
                            let pos = collapser.get_parent_pos(index)?;
                            let UndoData::Island(island) = collapser.get_dependend_undo_data_mut(index, Identifier::Island)? 
                            else { unreachable!() };
                            
                            let mut tree_grid = self.tree_grid.clone();
                            tree_grid.offset += pos.as_ivec3();
                            island.union.add_shared_grid(tree_grid);

                            info!("Tree Pos: {pos}");
                        },
                        Identifier::RiverNode => {
                            let pos = collapser.get_parent_pos(index)?;
                            let UndoData::Island(island) = collapser.get_dependend_undo_data_mut(index, Identifier::Island)?
                            else { unreachable!() };

                            island.union.add_sphere(pos, 10.0);

                            info!("River Pos: {pos}");
                        }
                        Identifier::IslandDone => {

                            let pos = collapser.get_dependend_pos(index, Identifier::IslandPositions, Identifier::Island)?;
                            let UndoData::Island(island) = collapser.get_dependend_undo_data_mut(index, Identifier::Island)?
                            else { unreachable!() };

                            island.union.calculate_bounds();

                            let active_key = self.dag.add_pos_query_volume(&island.union)?;
                            let scene_object_key = scene.add_dag_object(
                                Mat4::from_scale_rotation_translation(
                                    Vec3::ONE,
                                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                                    Vec3::from(pos) / METERS_PER_SHADER_UNIT as f32
                                ), 
                                self.scene_dag_key,
                                self.dag.get_entry(active_key),
                            ).result_async().await;

                            collapser.set_undo_data(index, UndoData::IslandDone(IslandDone {
                                scene_key: scene_object_key,
                                dag_key: active_key,
                            }));
                        }
                        _ => unreachable!()
                    }

                },
                CollapseOperation::Undo { identifier , undo_data} => {
                    info!("Undo {:?}", identifier);

                    match identifier {
                        Identifier::IslandDone => {
                            let UndoData::IslandDone(island_done) = undo_data
                            else { continue; };

                            scene.remove_object(island_done.scene_key);
                        },
                        _ => {}
                    }
                },
                CollapseOperation::None => {},
            } 
        }

        Ok(ticked)
    }
}

impl IslandUpdateData {
    pub fn new(camera: &Camera) -> Self {
        Self {
            pos: camera.get_position_in_meters(),
        }
    }
}

