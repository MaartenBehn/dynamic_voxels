use std::{iter};

use egui_snarl::{InPinId, NodeId};
use itertools::Itertools;
use octa_force::{anyhow::{anyhow, bail}, OctaResult};

use crate::{csg::csg_tree::tree::CSGTree, model::{composer::nodes::ComposeNodeType, generation::{builder::{BuilderAmmount, BuilderNode, ModelSynthesisBuilder}, template::{NodeTemplateValue, TemplateAmmountN, TemplateTree}, traits::{ModelGenerationTypes, BU, IT}}}, util::math_config::{Float2D, Float3D}};

use super::{data_type::ComposeDataType, nodes::ComposeNode, ModelComposer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComposerIdentifier {
    node_id: NodeId,
}

#[derive(Clone, Debug, Default)]
pub enum ComposerUndoData {
    #[default]
    None,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ComposerGenerationTypes {}
impl IT for ComposerIdentifier {}
impl BU for ComposerUndoData {}
impl ModelGenerationTypes for ComposerGenerationTypes {
    type Identifier = ComposerIdentifier;
    type UndoData = ComposerUndoData;
    type Volume = CSGTree<(), Float3D, 3>;
    type Volume2D = CSGTree<(), Float2D, 2>;
}

impl ModelComposer {
    pub fn to_template(&self) -> TemplateTree<ComposerGenerationTypes> {

        let mut builder = ModelSynthesisBuilder::new();

        for node in self.snarl.nodes() {
            if node.t == ComposeNodeType::TemplateNumberSet {

                let depends = self.get_depends_of_input(node);
                let ammount_node = self.get_input_node_by_type(node, ComposeDataType::Ammount);
                let ammount = match ammount_node.t {
                    ComposeNodeType::OnePer => BuilderAmmount::OnePer(ComposerIdentifier::new(self.get_input_node_by_type(node, ComposeDataType::Identifier).id)),
                    ComposeNodeType::OneGlobal => BuilderAmmount::OneGlobal,
                    //ComposeNodeType::NPer => BuilderAmmount::OnePer(ComposerIdentifier::new(self.get_input_node_by_type(node, ComposeDataType::Identifier).id)),
                    ComposeNodeType::DefinedBy => todo!(),
                    _ => unreachable!(),
                };


                builder.nodes.push(BuilderNode {
                    identifier: ComposerIdentifier{node_id: node.id},
                    value: NodeTemplateValue::NumberSetHook,
                    restricts: vec![],
                    depends,
                    knows: vec![],
                    ammount,
                });
            }
        }

        TemplateTree::new_from_builder(&builder)
    }

    fn get_depends_of_input(&self, node: &ComposeNode) -> Vec<ComposerIdentifier>  {
        node.inputs.iter()
            .enumerate()
            .map(|(i, input)| {
                self.snarl.in_pin(InPinId{ node: node.id, input: i }).remotes.into_iter()
                    .map(|out_pin_id| {
                        let input_node = self.snarl.get_node(out_pin_id.node).expect("Node of remote not found");

                        match input_node.t {
                            ComposeNodeType::Number
                            | ComposeNodeType::Position2D
                            | ComposeNodeType::Position3D
                            | ComposeNodeType::NumberRange
                            | ComposeNodeType::GridInVolume
                            | ComposeNodeType::GridOnPlane
                            | ComposeNodeType::Path
                            | ComposeNodeType::EmpytVolume2D
                            | ComposeNodeType::EmpytVolume3D
                            | ComposeNodeType::Sphere
                            | ComposeNodeType::Circle
                            | ComposeNodeType::Box
                            | ComposeNodeType::VoxelObject
                            | ComposeNodeType::UnionVolume2D
                            | ComposeNodeType::UnionVolume3D
                            | ComposeNodeType::CutVolume2D
                            | ComposeNodeType::CutVolume3D
                            | ComposeNodeType::SphereUnion
                            | ComposeNodeType::CircleUnion
                            | ComposeNodeType::VoxelObjectUnion
                            | ComposeNodeType::OnePer
                            | ComposeNodeType::OneGlobal
                            | ComposeNodeType::NPer
                            | ComposeNodeType::DefinedBy
                            | ComposeNodeType::PlayerPosition => self.get_depends_of_input(node),

                            // Template Nodes
                            ComposeNodeType::TemplatePositionSet 
                            | ComposeNodeType::TemplateNumberSet
                            | ComposeNodeType::BuildObject => vec![ComposerIdentifier { node_id: node.id }],

                        }
                    })
                    .flatten()
            })
            .flatten()
            .collect_vec()
    }

    fn get_input_node_by_type(&self, node: &ComposeNode, t: ComposeDataType) -> &ComposeNode {
        let index = node.inputs.iter()
            .position(|i|  i.data_type == t)
            .expect(&format!("Node {:?} input of type {:?}", node.t, t));

        let remotes = self.snarl.in_pin(InPinId{ node: node.id, input: index }).remotes;
        if remotes.is_empty() {
            panic!("No {:?} node connected to {:?}", t, node.t);
        }

        if remotes.len() >= 2 {
            panic!("More than one {:?} node connected to {:?}", t, node.t);
        }

        let remote_node = self.snarl.get_node(remotes[0].node).expect("Node of remote not found");
        remote_node
    }
}

impl ComposerIdentifier {
    pub fn new(node_id: NodeId) -> Self {
        ComposerIdentifier { node_id: node_id }
    }
} 

impl Default for ComposerIdentifier {
    fn default() -> Self {
        Self { node_id: NodeId(usize::MAX) }
    }
}
