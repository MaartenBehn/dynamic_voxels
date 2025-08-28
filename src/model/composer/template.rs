use std::{iter, ops::RangeBounds};

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::Itertools;
use octa_force::glam::Vec3;
use crate::util::number::Nu;

use crate::model::generation::{relative_path::RelativePathTree};

use super::pos_space::{GridVolumeData, PositionSpaceRule};
use super::{data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, number_space::NumberSpace, pos_space::PositionSpace, primitive::Number, ModelComposer};

pub type TemplateIndex = usize;
pub const TEMPLATE_INDEX_ROOT: TemplateIndex = 0;
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct ComposeTemplate {
    pub nodes: Vec<TemplateNode>,
    pub max_level: usize,
}

#[derive(Debug, Clone)]
pub enum ComposeTemplateValue {
    None,
    NumberSpace(NumberSpace),
    PositionSpace(PositionSpace),
    Object()
}

#[derive(Debug, Clone)]
pub struct TemplateNode {
    pub identifier: NodeId,
    pub index: TemplateIndex,
    pub value: ComposeTemplateValue,
    pub restricts: Vec<TemplateIndex>,
    pub depends: Vec<TemplateIndex>,
    pub dependend: Vec<TemplateIndex>,
    pub level: usize,
    pub defines_n: Vec<TemplateAmmountN>,
    pub defines_by_value: Vec<TemplateAmmountValue>,
}

#[derive(Debug, Clone)]
pub struct TemplateAmmountN {
    pub ammount: usize,
    pub template_index: TemplateIndex,
    pub dependecy_tree: RelativePathTree,
}

#[derive(Debug, Clone)]
pub struct TemplateAmmountValue {
    pub template_index: TemplateIndex,
    pub dependecy_tree: RelativePathTree,
}

impl ComposeTemplate {
    pub fn new(composer: &ModelComposer) -> ComposeTemplate {
        let nodes = composer.snarl.nodes()
            .map(|node| {
                match &node.t {
                    ComposeNodeType::TemplateNumberSet => {
                        let space = composer.make_number_space(
                            composer.get_input_node_by_type(node, ComposeDataType::NumberSpace));

                        Some((node, ComposeTemplateValue::NumberSpace(space)))
                    },
                    ComposeNodeType::TemplatePositionSet => {
                        let space = composer.make_pos_space(
                            composer.get_input_node_by_type(node, ComposeDataType::PositionSpace));

                        Some((node, ComposeTemplateValue::PositionSpace(space)))
                    },

                    _ => {None}
                }
            })
            .flatten()
            .enumerate()
            .map(|(i, (node, value))| {

            })
            .collect_vec();
        
    }
}

impl ModelComposer {
    pub fn get_input_index_by_type(&self, node: &ComposeNode, t: ComposeDataType) -> usize {
        node.inputs.iter()
            .position(|i|  i.data_type == t)
            .expect(&format!("Node {:?} input of type {:?}", node.t, t))
    }

    pub fn get_input_node_by_type(&self, node: &ComposeNode, t: ComposeDataType) -> OutPinId {
        self.get_input_node_by_index(node, self.get_input_index_by_type(node, t))
    }

    pub fn get_input_node_by_index(&self, node: &ComposeNode, index: usize) -> OutPinId {
        let remotes = self.snarl.in_pin(InPinId{ node: node.id, input: index }).remotes;
        if remotes.is_empty() {
            panic!("No node connected to {:?}", node.t);
        }

        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", node.t);
        }

        remotes[0]
    }
}
