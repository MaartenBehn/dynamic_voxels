use std::{iter, ops::RangeBounds};

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::Itertools;
use octa_force::glam::Vec3;
use smallvec::{SmallVec, smallvec};
use crate::model::composer::dependency_tree::DependencyTree;
use crate::util::number::Nu;

use crate::model::generation::{relative_path::RelativePathTree};

use super::ammount::Ammount;
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
    pub node_id: NodeId,
    pub index: TemplateIndex,
    pub value: ComposeTemplateValue,
    pub depends: SmallVec<[TemplateIndex; 4]>,
    pub dependend: SmallVec<[TemplateIndex; 4]>,
    pub level: usize,
    pub defines: SmallVec<[Ammount; 4]>,
}

impl ComposeTemplate {
    pub fn new(composer: &ModelComposer) -> ComposeTemplate {
        let mut nodes = vec![
            TemplateNode {
                node_id: NodeId(usize::MAX),
                index: 0,
                value: ComposeTemplateValue::None,
                depends: smallvec![],
                dependend: smallvec![],
                level: 0,
                defines: smallvec![],
            }]; 

        nodes.extend(composer.snarl.nodes()
            .map(|node| {
                match &node.t {
                    ComposeNodeType::TemplateNumberSet 
                    | ComposeNodeType::TemplatePositionSet => Some(node.id),
                    _ => {None}
                }
            })
            .flatten()
            .enumerate()
            .map(|(i, node_id)| {
                TemplateNode {
                    node_id: node_id,
                    index: i + 1,
                    value: ComposeTemplateValue::None,
                    depends: smallvec![],
                    dependend: smallvec![],
                    level: 0,
                    defines: smallvec![],
                }
            }));

        let mut template = ComposeTemplate {
            nodes,
            max_level: 0,
        };

        // Values Depends and Dependend
        for i in 1..template.nodes.len() {
            let node =  &template.nodes[i]; 
            let composer_node = composer.snarl.get_node(node.node_id)
                .expect("Composer Node for Template not found");

            let (ammount, parent_index) = composer.make_ammount(
                composer.get_input_node_by_type(composer_node,
                ComposeDataType::Ammount), i, &template);

            let (mut depends , value): (SmallVec<[TemplateIndex; 4]>, ComposeTemplateValue) = match &composer_node.t { 
                ComposeNodeType::TemplatePositionSet => {
                    let space = composer.make_pos_space(
                        composer.get_input_node_by_type(composer_node, ComposeDataType::PositionSpace),
                        &template);
                    (
                        space.get_dependend_template_nodes().collect(),
                        ComposeTemplateValue::PositionSpace(space)
                    )
                },
                ComposeNodeType::TemplateNumberSet => {
                    let space = composer.make_number_space(
                        composer.get_input_node_by_type(composer_node, ComposeDataType::NumberSpace),
                        &template);
                    
                    (
                        space.get_dependend_template_nodes().collect(),
                        ComposeTemplateValue::NumberSpace(space)
                    )
                },
                _ => unreachable!()
            };

            depends.push(parent_index);
            depends.sort();
            depends.dedup();
            
            let parent_node = &mut template.nodes[parent_index];
            parent_node.defines.push(ammount);
            parent_node.dependend.push(i);

            for depend in depends.iter() {
                let dependend_node = &mut template.nodes[*depend];
                if !dependend_node.dependend.contains(&i) {
                    dependend_node.dependend.push(i);
                }
            }

            let node =  &mut template.nodes[i]; 
            node.depends = depends;
            node.value = value;
        }

        // Levels and dependency_tree
        for i in 0..template.nodes.len() {

            if template.nodes[i].level == 0 {
                template.set_level_of_node(i);
            }

            for j in 0..template.nodes[i].defines.len() {
                let new_index = template.nodes[i].defines[j].template_index; 
                let new_node = &template.nodes[new_index];

                template.nodes[i].defines[j].dependecy_tree = DependencyTree::new(
                    &template, 
                    i,
                    &new_node.depends,);
            }
        }

        template
    }

    fn set_level_of_node(&mut self, index: usize) -> usize {
        let node = &self.nodes[index];

        let mut max_level = 0;
        for index in node.depends.to_owned().iter() {
            let mut level = self.nodes[*index].level; 

            if level == 0 {
                level = self.set_level_of_node(*index);
            } 

            max_level = max_level.max(level);
        }

        let node_level = max_level + 1;
        self.nodes[index].level = node_level;
        self.max_level = self.max_level.max(node_level);

        node_level
    } 

    pub fn get_index_by_out_pin(&self, pin: OutPinId) -> TemplateIndex {
        self.nodes.iter().position(|n| n.node_id == pin.node).expect("No Template Node for node id found")
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
