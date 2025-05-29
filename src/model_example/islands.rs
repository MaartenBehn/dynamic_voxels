
use octa_force::{glam::{vec3, Mat4, Quat, Vec3}, log::{error, info}, OctaResult};

use crate::{model_synthesis::{builder::{BuilderAmmount, BuilderValue, ModelSynthesisBuilder, IT}, collapse::CollapseOperation, collapser_data::CollapserData, pos_set::{PositionSet, PositionSetRule}, template::TemplateTree}, slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey}, state_saver::State, vec_csg_tree::tree::{VecCSGNode, VecCSGTree, VOXEL_SIZE}};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Identifier {
    #[default]
    None,
    MinIslandDistance,
    IslandRadius,
    BeachWidth,
    IslandRoot,
}
impl IT for Identifier {}

#[derive(Clone, Debug)]
pub struct IslandsState {
    pub template: TemplateTree<Identifier>,
}

impl IslandsState {
    pub fn new() -> Self {
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
                    VecCSGTree::new_sphere(Vec3::ZERO, 1.5), 
                    PositionSetRule::Grid { spacing: 0.1 })))
            });

        let template = wfc_builder.build_template();

        Self {
            template,
        }
    }
}

impl State for IslandsState {
    fn tick_state(&mut self) -> OctaResult<bool> {
        let mut ticked = false;

        /*
        let mut collapser = self.collapser.take().unwrap().into_collapser(&self.template);
        if let Some((operation, collapser)) = collapser.next()? {
            ticked = true;

            match operation {
                CollapseOperation::CollapsePos{ index  } => {
                                  
                },
                CollapseOperation::CollapseBuild{ index, identifier, .. } => {
                           
                }, 
                CollapseOperation::Undo { identifier , undo_data} => {
                    info!("Undo {:?}", identifier);

                },
                CollapseOperation::None => {},
            } 
        }
        */

        Ok(ticked)
    }
}

