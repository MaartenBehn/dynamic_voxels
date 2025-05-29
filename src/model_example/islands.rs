
use octa_force::{glam::{vec3, Mat4, Quat, Vec3}, log::{error, info}, OctaResult};

use crate::{model_synthesis::{builder::{BuilderAmmount, BuilderValue, ModelSynthesisBuilder, IT}, collapse::{CollapseOperation, Collapser}, collapser_data::CollapserData, pos_set::{PositionSet, PositionSetRule}, template::TemplateTree}, slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey}, state_saver::State, vec_csg_tree::tree::{VecCSGNode, VecCSGTree, VOXEL_SIZE}};

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

#[derive(Clone, Debug)]
pub struct IslandsState {
    pub template: TemplateTree<Identifier, VecCSGTree>,
    pub collapser: Option<CollapserData<Identifier, SlotMapCSGTreeKey, VecCSGTree>>,
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
                    VecCSGTree::new_disk(Vec3::ZERO, 20.0, 0.1), 
                    PositionSetRule::Grid { spacing: 0.1 })))
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

        Self {
            template,
            collapser: Some(collapser),
        }
    }

    pub fn run(&mut self) -> OctaResult<()> {
        let mut collapser = self.collapser.take().unwrap().into_collapser(&self.template);
        while let Some((hook, collapser)) = collapser.next()? {
            Self::handle_hook(collapser, hook);
        }

        self.collapser = Some(collapser.into_data());

        Ok(())
    }

    pub fn handle_hook( 
        collapser: &mut Collapser<Identifier, SlotMapCSGTreeKey, VecCSGTree>,
        hook: CollapseOperation<Identifier, SlotMapCSGTreeKey>, 
    ) {
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
                info!("Build Hook: {pos}")
            },
            CollapseOperation::Undo { identifier , undo_data} => {
                #[cfg(not(feature = "profile_islands"))]
                info!("Undo {:?}", identifier);

            },
            CollapseOperation::None => {},
        } 
    }
}

impl State for IslandsState {
    fn tick_state(&mut self) -> OctaResult<bool> {
        let mut ticked = false;

        let mut collapser = self.collapser.take().unwrap().into_collapser(&self.template);
        if let Some((hook, collapser)) = collapser.next()? {
            ticked = true;
            Self::handle_hook(collapser, hook);
        }

        self.collapser = Some(collapser.into_data());
        
        Ok(ticked)
    }
}

