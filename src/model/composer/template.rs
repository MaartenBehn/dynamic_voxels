use std::{iter, ops::RangeBounds};

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::Itertools;
use octa_force::glam::Vec3;
use smallvec::{SmallVec, smallvec};
use crate::model::composer::dependency_tree::DependencyTree;
use crate::util::number::Nu;

use crate::model::generation::{relative_path::RelativePathTree};
use crate::util::vector::Ve;

use super::ammount::Ammount;
use super::build::{GetTemplateValueArgs, TemplateValueTrait, BS};
use super::position_space::PositionSpaceTemplate;
use super::{data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, number_space::NumberSpaceTemplate, ModelComposer};

pub type TemplateIndex = usize;
pub type OutputIndex = usize;
pub const TEMPLATE_INDEX_ROOT: TemplateIndex = 0;
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct ComposeTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub nodes: Vec<TemplateNode<V2, V3, T, B>>,
    pub max_level: usize,
}

#[derive(Debug, Clone)]
pub enum ComposeTemplateValue<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    None,
    NumberSpace(NumberSpaceTemplate<V2, V3, T>),
    PositionSpace2D(PositionSpaceTemplate<V2, V2, V3, T, 2>),
    PositionSpace3D(PositionSpaceTemplate<V3, V2, V3, T, 3>),
    Build(B::TemplateValue)
}

#[derive(Debug, Clone)]
pub struct TemplateNode<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub node_id: NodeId,
    pub index: TemplateIndex,
    pub value: ComposeTemplateValue<V2, V3, T, B>,
    pub depends: SmallVec<[TemplateIndex; 4]>,
    pub dependend: SmallVec<[TemplateIndex; 4]>,
    pub level: usize,
    pub defines: SmallVec<[Ammount<V2, V3, T>; 4]>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeTemplate<V2, V3, T, B> {
    pub fn empty() -> Self {
        Self {
            nodes:  vec![
                TemplateNode {
                    node_id: NodeId(usize::MAX),
                    index: 0,
                    value: ComposeTemplateValue::None,
                    depends: smallvec![],
                    dependend: smallvec![],
                    level: 1,
                    defines: smallvec![],
                }],
            max_level: 1,
        }
    }

    pub fn new(composer: &ModelComposer<V2, V3, T, B>) -> Self {
        let mut nodes = vec![
            TemplateNode {
                node_id: NodeId(usize::MAX),
                index: 0,
                value: ComposeTemplateValue::None,
                depends: smallvec![],
                dependend: smallvec![],
                level: 1,
                defines: smallvec![],
            }]; 

        nodes.extend(composer.snarl.nodes()
            .map(|node| {
                match &node.t {
                    ComposeNodeType::TemplateNumberSet 
                    | ComposeNodeType::TemplatePositionSet2D
                    | ComposeNodeType::TemplatePositionSet3D => Some(node.id),
                    ComposeNodeType::Build(t) => if B::is_template_node(t) {
                        Some(node.id) 
                    } else {
                        None
                    }
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
            max_level: 1,
        };

        dbg!(&template);

        // Values Depends and Dependend
        for i in 1..template.nodes.len() {
            let template_node =  &template.nodes[i]; 
            let composer_node = composer.snarl.get_node(template_node.node_id)
                .expect("Composer Node for Template not found");

            let (ammount, parent_index) = composer.make_ammount(
                composer.get_input_remote_pin_by_type(composer_node,
                ComposeDataType::Ammount), i, &template);

            let (mut depends , value): (SmallVec<[TemplateIndex; 4]>, ComposeTemplateValue<V2, V3, T, B>) = match &composer_node.t { 
                ComposeNodeType::TemplatePositionSet2D => {
                    let space = composer.make_pos_space(
                        composer.get_input_remote_pin_by_type(composer_node, ComposeDataType::PositionSpace2D),
                        i, &template);
                    (
                        space.get_dependend_template_nodes().collect(),
                        ComposeTemplateValue::PositionSpace2D(space)
                    )
                },
                ComposeNodeType::TemplatePositionSet3D => {
                    let space = composer.make_pos_space(
                        composer.get_input_remote_pin_by_type(composer_node, ComposeDataType::PositionSpace3D),
                        i, &template);
                    (
                        space.get_dependend_template_nodes().collect(),
                        ComposeTemplateValue::PositionSpace3D(space)
                    )
                },
                ComposeNodeType::TemplateNumberSet => {
                    let space = composer.make_number_space(
                        composer.get_input_remote_pin_by_type(composer_node, ComposeDataType::NumberSpace),
                        i, &template);
                    
                    (
                        space.get_dependend_template_nodes().collect(),
                        ComposeTemplateValue::NumberSpace(space)
                    )
                },
                ComposeNodeType::Build(t) => {
                    let value = B::get_template_value(GetTemplateValueArgs { 
                        compose_type: t, 
                        composer_node, 
                        composer: &composer, 
                        template: &template,
                        building_template_index: i, 
                    });

                    let depends = value.get_dependend_template_nodes();

                    (depends, ComposeTemplateValue::Build(value))
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

        dbg!(&template);

        // Levels and dependency_tree
        for i in 0..template.nodes.len() {

            if template.nodes[i].level == 0 {
                template.set_level_of_node(i, i);
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

        dbg!(&template);

        template
    }

    fn set_level_of_node(&mut self, index: usize, index_self: usize) -> usize {
        let node: &TemplateNode<V2, V3, T, B> = &mut self.nodes[index];

        let mut max_level = 0;
        for index in node.depends.to_owned().iter().rev() {
            if *index == index_self {
                let node = &mut self.nodes[*index];

                match &mut node.value {
                    ComposeTemplateValue::NumberSpace(number_space_template) => {
                        number_space_template.cut_loop(index_self);
                    },
                    ComposeTemplateValue::PositionSpace2D(position_space_template) => {
                        position_space_template.cut_loop(index_self)
                    },
                    ComposeTemplateValue::PositionSpace3D(position_space_template) => {
                        position_space_template.cut_loop(index_self);
                    },
                    _ => {} 
                }

                node.depends.swap_remove(*index);
                continue;
            }

            let mut level = self.nodes[*index].level; 

            if level == 0 {
                level = self.set_level_of_node(*index, index_self);
            } 

            max_level = max_level.max(level);
        }

        let node_level = max_level + 1;
        self.nodes[index].level = node_level;
        self.max_level = self.max_level.max(node_level);

        node_level
    }

    pub fn get_index_by_out_pin(&self, pin: OutPinId) -> TemplateIndex {
        self.nodes.iter()
            .position(|n| n.node_id == pin.node)
            .expect("No Template Node for node id found")
    }

    pub fn get_index_by_in_pin(&self, pin: InPinId) -> TemplateIndex {
        self.nodes.iter()
            .position(|n| n.node_id == pin.node)
            .expect("No Template Node for node id found")
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn get_input_pin_index_by_type(&self, node: &ComposeNode<B::ComposeType>, t: ComposeDataType) -> usize {
        node.inputs.iter()
            .position(|i|  i.data_type == t)
            .expect(&format!("No Node {:?} input of type {:?}", node.t, t))
    }
 
    pub fn get_input_remote_pin_by_type(&self, node: &ComposeNode<B::ComposeType>, t: ComposeDataType) -> OutPinId {
        self.get_input_remote_pin_by_index(node, self.get_input_pin_index_by_type(node, t))
    }

    pub fn get_input_remote_pin_by_index(&self, node: &ComposeNode<B::ComposeType>, index: usize) -> OutPinId {
        let remotes = self.snarl.in_pin(InPinId{ node: node.id, input: index }).remotes;
        if remotes.is_empty() {
            panic!("No node connected to {:?}", node.t);
        }

        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", node.t);
        }

        remotes[0]
    }

    pub fn get_output_pin_index_by_type(&self, node: &ComposeNode<B::ComposeType>, t: ComposeDataType) -> usize {
        node.outputs.iter()
            .position(|i|  i.data_type == t)
            .expect(&format!("No Node {:?} output of type {:?}", node.t, t))
    }

    pub fn get_output_first_remote_pin_by_type(&self, node: &ComposeNode<B::ComposeType>, t: ComposeDataType) -> InPinId {
        self.get_output_first_remote_pin_by_index(node, self.get_output_pin_index_by_type(node, t))
    }

    pub fn get_output_first_remote_pin_by_index(&self, node: &ComposeNode<B::ComposeType>, index: usize) -> InPinId {
        let remotes = self.snarl.out_pin(OutPinId{ node: node.id, output: index }).remotes;
        if remotes.is_empty() {
            panic!("No output node connected to {:?}", node.t);
        }

        remotes[0]
    }
}
