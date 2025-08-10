
use std::sync::Arc;

use nalgebra::IsDynamic;
use octa_force::{camera::Camera, glam::{vec3, EulerRot, Mat4, Quat, Vec2, Vec3, Vec3A, Vec3Swizzles}, log::{error, info}, vulkan::{Context, Swapchain}, OctaResult};
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{csg::{csg_tree::tree::{CSGNode, CSGTree}, csg_tree_2d::tree::CSGTree2D, fast_query_csg_tree::tree::FastQueryCSGTree}, model::generation::{builder::{BuilderAmmount, BuilderValue, ModelSynthesisBuilder}, collapse::{CollapseOperation, Collapser}, pos_set::{PositionSet, PositionSetRule}, template::TemplateTree, traits::{ModelGenerationTypes, BU, IT}}, scene::{dag64::DAG64SceneObject, renderer::SceneRenderer, Scene, SceneObjectData, SceneObjectKey}, util::aabb3d::AABB, volume::{magica_voxel::MagicaVoxelModel, VolumeBoundsI, VolumeQureyPosValid}, voxel::{dag64::{DAG64EntryKey, VoxelDAG64}, grid::{shared::SharedVoxelGrid, VoxelGrid}, renderer::palette::Palette}, METERS_PER_SHADER_UNIT};

const COLLAPSES_PER_TICK: usize = 100;

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
    TreeBuild,
}


#[derive(Clone, Copy, Debug, Default)]
pub struct IslandGenerationTypes {}
impl IT for Identifier {}
impl BU for Island {}
impl ModelGenerationTypes for IslandGenerationTypes {
    type Identifier = Identifier;
    type UndoData = Island;
    type Volume = FastQueryCSGTree<()>;
    type Volume2D = CSGTree2D<()>;
}

#[derive(Clone, Debug)]
pub struct Islands {
    pub template: TemplateTree<IslandGenerationTypes>,
    pub collapser: Collapser<IslandGenerationTypes>,

    pub dag: Arc<Mutex<VoxelDAG64>>,
    pub last_pos: Vec3,
    pub tree_grid: SharedVoxelGrid,
}

#[derive(Clone, Debug, Default)]
pub struct Island {
    pub csg: CSGTree<u8>,
    pub scene_key: SceneObjectKey,
    pub dag_key: DAG64EntryKey,
}


impl Islands {
    pub fn new(palette: &mut Palette) -> OctaResult<Self> {

        let mut dag = VoxelDAG64::new(1000000, 1000000);
        dag.print_memory_info();
        
        let tree_model = MagicaVoxelModel::new("./assets/Fall_Tree.vox")?;
        let tree_grid: SharedVoxelGrid = tree_model.into_grid(palette)?.into();
         
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
                    100.0 )
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
            
            .build(Identifier::TreeBuild, |b|{b
                .ammount(BuilderAmmount::DefinedBy(Identifier::TreePositions))
                .depends(Identifier::TreePositions)
                .depends(Identifier::IslandBuild)
            });

        let template = wfc_builder.build_template();

        let collapser = template.get_collaper();

        Ok(Self {
            template,
            collapser,
            dag: Arc::new(Mutex::new(dag)),
            last_pos: Vec3::ZERO,
            tree_grid,
        })
    }

    pub fn update(&mut self, camera: &Camera) -> OctaResult<()> {

        let mut new_pos = camera.get_position_in_meters();
        new_pos.z = 0.0;

        if new_pos == self.last_pos {
            return Ok(());
        }
        self.last_pos = new_pos;

        let island_volume = CSGTree::new_sphere(new_pos, 200.0); 
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
        let mut i = 0;

        while let Some((hook, collapser)) 
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

                            //let mut pos = collapser.get_dependend_pos(index, Identifier::IslandPositions, Identifier::IslandBuild);
                            let tree_volume = CSGTree2D::new_circle(Vec2::ZERO, 10.0); 

                            collapser.set_position_set_value(index, PositionSet::new_grid_on_plane(
                                tree_volume,
                                100.0, 
                                0.0)
                            );    
                        },
                        _ => unreachable!()
                    }
                },
                CollapseOperation::BuildHook { index, identifier } => {

                    match identifier {
                        Identifier::IslandBuild => {

                            let pos = collapser.get_parent_pos(index);
                            info!("Island Pos: {pos}");

                            let csg = CSGTree::new_disk(Vec3::ZERO, 100.0, 10.0);
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

                            let island = Island {
                                csg,
                                scene_key: scene_object_key,
                                dag_key: active_key,
                            };

                            collapser.set_undo_data(index, island)?;

                        },
                        Identifier::TreeBuild => {
                            let pos = collapser.get_parent_pos(index);
                            let island = collapser.get_dependend_undo_data_mut(index, Identifier::IslandBuild);
                            island.csg.append_node_with_union(CSGNode::new_shared_grid(self.tree_grid.clone()));
                            island.csg.calculate_bounds();

                            let active_key = self.dag.lock().update_pos_query_volume(&island.csg, island.dag_key)?;
                            island.dag_key = active_key;
                            scene.set_dag64_entry_key(island.scene_key, active_key)?;
                            
                            info!("Tree Pos: {pos}");
                        },
                        _ => unreachable!()
                    }

                },
                CollapseOperation::Undo { identifier , undo_data} => {
                    info!("Undo {:?}", identifier);

                    match identifier {
                        Identifier::IslandBuild => {
                            scene.remove_object(undo_data.scene_key)?;
                        },
                        Identifier::TreeBuild => {}
                        Identifier::TreePositions => {}
                        _ => unreachable!()
                    }
                                    },
                CollapseOperation::None => {},
            } 

            i += 1;
            if i > COLLAPSES_PER_TICK {
                break;
            }
        }

        if !ticked {
            //info!("Idle");
        }

        Ok(ticked)
    }
}

