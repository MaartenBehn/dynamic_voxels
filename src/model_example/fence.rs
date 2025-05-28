
use octa_force::{glam::{vec3, Mat4, Quat, Vec3}, log::{error, info}, OctaResult};

use crate::{model_synthesis::{builder::{BuilderAmmount, BuilderValue, ModelSynthesisBuilder, IT}, collapse::CollapseOperation, collapser_data::CollapserData, template::TemplateTree}, slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey}, state_saver::State, vec_csg_tree::tree::{VecCSGTree, VOXEL_SIZE}};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Identifier {
    #[default]
    None,
    Fence,
    FencePost,
    PlankSetting,
    FencePlanks,

    PostNumber,
    PostHeight,
    PostDistance,
    PostPos,
    PlankNumber,
    PlankDistance,
}
impl IT for Identifier {}

#[derive(Clone, Debug)]
pub struct FenceState {
    pub template: TemplateTree<Identifier>,
    pub collapser: Option<CollapserData<Identifier, SlotMapCSGTreeKey>>,
    pub pos: Vec3,
    pub start_pos: Vec3,
    pub csg: Option<SlotMapCSGTree>,
}

impl FenceState {
    pub fn new() -> Self {
        let mut wfc_builder: ModelSynthesisBuilder<Identifier> = ModelSynthesisBuilder::new()
            .groupe(Identifier::Fence, |b| {b})

            .number_range(Identifier::PostHeight, |b|{b
                .ammount(BuilderAmmount::OnePer(Identifier::Fence))
                .value(BuilderValue::Const(3..=8))
            })

            .number_range(Identifier::PostDistance, |b|{b
                .ammount(BuilderAmmount::OnePer(Identifier::Fence))
                .value(BuilderValue::Const(2..=5))
            })

            .number_range(Identifier::PostNumber, |b|{b
                .ammount(BuilderAmmount::OnePer(Identifier::Fence))
                .value(BuilderValue::Const(5..=10))
            })

            .pos(Identifier::PostPos, |b| {b
                .ammount(BuilderAmmount::DefinedBy(Identifier::PostNumber))
                .depends(Identifier::PostDistance)
            })

            .build(Identifier::FencePost, |b| {b
                .ammount(BuilderAmmount::OnePer(Identifier::PostPos))
                .depends(Identifier::PostHeight)
                .depends(Identifier::PostDistance)
            })

            .number_range(Identifier::PlankNumber, |b|{b
                .ammount(BuilderAmmount::OnePer(Identifier::Fence))
                .value(BuilderValue::Const(3..=4))
            })

            .number_range(Identifier::PlankDistance, |b|{b
                .ammount(BuilderAmmount::OnePer(Identifier::Fence))
                .value(BuilderValue::Const(2..=5))
            })

            .build(Identifier::FencePlanks, |b|{b
                .ammount(BuilderAmmount::OnePer(Identifier::Fence))
                .depends(Identifier::PlankNumber)
                .depends(Identifier::PlankDistance)
                .depends(Identifier::PostHeight)
                .knows(Identifier::PostDistance)
            });
        
        let template = wfc_builder.build_template();

        let pos = vec3(1.0, 1.0, 1.0);
        let start_pos = pos;
        let csg = None;

        let collapser = template.get_collaper().into_data();

        Self {
            collapser: Some(collapser),
            template,
            pos, 
            start_pos, 
            csg,
        }
    }
}

impl State for FenceState {
    fn tick_state(&mut self) -> OctaResult<bool> {
        let mut ticked = false;

        let mut collapser = self.collapser.take().unwrap().into_collapser(&self.template);
        if let Some((operation, collapser)) = collapser.next()? {
            ticked = true;

            match operation {
                CollapseOperation::CollapsePos{ index  } => {
                    let dist = collapser.get_dependend_number(index, Identifier::PostDistance);

                    let pos_data = collapser.get_pos_mut(index);
                    *pos_data = self.pos;

                    info!("{:?} Pos: {}", index, self.pos);

                    self.pos += Vec3::X * dist as f32;
                },
                CollapseOperation::CollapseBuild{ index, identifier, .. } => {
                    match identifier {
                        Identifier::FencePost => {
                            let height = collapser.get_dependend_number(index, Identifier::PostHeight);
                            let distance = collapser.get_dependend_number(index, Identifier::PostDistance);
                            let pos_value = collapser.get_dependend_pos(index, Identifier::PostPos);

                            let pos = pos_value + Vec3::Z * (height as f32) * 0.5;

                            let csg_node = SlotMapCSGNode::new(SlotMapCSGNodeData::Box(
                                Mat4::from_scale_rotation_translation(
                                    vec3(0.5, 0.5, height as f32) * VOXEL_SIZE, 
                                    Quat::IDENTITY, 
                                    pos * VOXEL_SIZE
                                ),
                                1,
                            ));
                            info!("{:?} Build: {:?}: {}", index, identifier, pos);

                            let csg_index = if self.csg.is_none() {
                                self.csg = Some(SlotMapCSGTree::from_node(csg_node));
                                self.csg.as_ref().unwrap().root_node
                            } else {
                                self.csg.as_mut().unwrap().append_node_with_union(csg_node)
                            };

                            collapser.set_undo_data(index, csg_index)?;
                        }
                        Identifier::FencePlanks => {
                            let plank_number = collapser.get_dependend_number(index, Identifier::PlankNumber);
                            let plank_distance = collapser.get_dependend_number(index, Identifier::PlankDistance);
                            let fence_height = collapser.get_dependend_number(index, Identifier::PostHeight);
                            let post_distance = collapser.get_known_number(index, Identifier::PostDistance);

                            if plank_number * plank_distance > fence_height {
                                collapser.collapse_failed(index)?;
                                
                            } else {
                                let pos = self.pos - Vec3::X * post_distance as f32;
                                let plank_size = pos - self.start_pos;
                                let mut plank_pos = self.start_pos + plank_size * vec3(0.5, 1.0, 1.0);
                                let plank_scale = vec3(plank_size.x, 0.2, 0.2);
                                
                                for _ in 0..plank_number {
                                    plank_pos += Vec3::Z * plank_distance as f32;

                                    let mut node = SlotMapCSGNode::new(SlotMapCSGNodeData::Box(
                                        Mat4::from_scale_rotation_translation(
                                            plank_scale * VOXEL_SIZE, 
                                            Quat::IDENTITY, 
                                            plank_pos * VOXEL_SIZE
                                        ),
                                        1,
                                    ));
                                    if self.csg.is_none() {
                                        self.csg = Some(SlotMapCSGTree::from_node(node));
                                    } else {
                                        self.csg.as_mut().unwrap().append_node_with_union(node);
                                    }
                                } 
                            }  
                            
                                                    }
                        _ => error!("Build hook on wrong type")
                    }
                }, 
                CollapseOperation::Undo { identifier , undo_data} => {
                    info!("Undo {:?}", identifier);

                    match identifier {
                        Identifier::FencePost => {
                            self.csg.as_mut().unwrap().remove_node_as_child_of_union(undo_data)?;
                        },
                        Identifier::PostNumber => {
                            self.pos = vec3(1.0, 1.0, 1.0);
                        },
                        Identifier::PostPos => {
                            self.pos = vec3(1.0, 1.0, 1.0);
                        },
                        _ => {}
                    }
                },

                CollapseOperation::None => {},
            } 
        }

        self.collapser = Some(collapser.into_data());

        Ok(ticked)
    }
}

