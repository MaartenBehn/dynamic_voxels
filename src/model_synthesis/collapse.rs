use core::panic;
use std::{fmt::Debug, usize};

use fdg::nalgebra::base;
use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{glam::{vec3, IVec3, Vec3}, log::debug};

use crate::model_synthesis::{func_data::BuildFuncData, volume::PossibleVolume};

use super::builder::{AttributeTemplate, AttributeTemplateValue, Identifier, NodeTemplate, NumberRangeDefinesType, WFCBuilder};

#[derive(Debug, Clone)]
pub struct Node<U: Clone + Debug> {
    pub template_index: usize,
    pub number_attributes: Vec<usize>,
    pub pos_attributes: Vec<usize>,
    pub user_data: Option<U>,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct NumberAttribute {
    pub template_index: usize,
    pub node_index: usize,
    pub value: i32,
    pub perm_counter: usize,
}

#[derive(Debug, Clone)]
pub struct PosAttribute {
    pub template_index: usize,
    pub node_index: usize,
    pub value: Vec3,
}

impl<U: Clone + Debug, B: Clone + Debug> WFCBuilder<U, B> {
    pub fn collapse(self, build_data: &mut B) {

        let root_node_template = &self.nodes[0];
        let root_node = Node { 
            number_attributes: vec![], 
            pos_attributes: vec![], 
            template_index: 0,
            user_data: root_node_template.user_data.to_owned(),
            children: vec![],
        };

        let mut nodes = vec![root_node];
        let mut number_attributes = vec![];
        let mut pos_attributes = vec![];
        let mut possible_volumes = vec![];

        let mut pending_nodes = vec![0];
        let mut pending_number_attributes = vec![];
        let mut pending_pos_attributes = vec![];
        let mut pending_possible_volumes: Vec<usize> = vec![];

        let mut current_template_node = 0;
        loop {
            if let Some(current_attribute_index) = pending_number_attributes.pop() {
                let attribute: &mut NumberAttribute = &mut number_attributes[current_attribute_index];
                let attribute_template = &self.attributes[attribute.template_index];
                
                if attribute.perm_counter >= attribute_template.value.get_number_permutation().max() as usize {
                    
                }

                let value = attribute_template.value.get_number_permutation().get(attribute.perm_counter as _);
                attribute.perm_counter += 1;

                debug!("Value: {value}");

                attribute.value = value as i32 + attribute_template.value.get_number_min();

                if let NumberRangeDefinesType::Amount { of_node, .. } = attribute_template.value.get_number_defines() {
                     
                    for i in 0..attribute.value {
                        let node_template_index = self.get_node_index_by_identifier(*of_node);
                        let node_template = &self.nodes[node_template_index];

                        let node = Node {
                            template_index: node_template_index,
                            number_attributes: vec![], 
                            pos_attributes: vec![],
                            user_data: node_template.user_data.to_owned(),
                            children: vec![],
                        };

                        let node_index = nodes.len();
                        nodes.push(node);
                        pending_nodes.push(node_index);

                        nodes[attribute.node_index].children.push(node_index);
                    }
                }
                    
            } else if let Some(current_attribute_index) = pending_pos_attributes.pop() {
                let attribute: &mut PosAttribute = &mut pos_attributes[current_attribute_index];
                let attribute_template = &self.attributes[attribute.template_index];

                
            } else if let Some(current_attribute_index) = pending_possible_volumes.pop() {
                let attribute: &mut PossibleVolume = &mut possible_volumes[current_attribute_index];
                let attribute_template = &self.attributes[attribute.template_index];

                
            } else if let Some(current_node_index) = pending_nodes.pop() {
                let node = &mut nodes[current_node_index];            
                let template_node = &self.nodes[node.template_index];
                
                for attribute_template_identifier in template_node.attributes.iter() {
                    let attribute_template_index = self.get_attribute_index_by_identifier(*attribute_template_identifier);
                    let attribute_template = &self.attributes[attribute_template_index];

                    match attribute_template.value {
                        AttributeTemplateValue::NumberRange { .. } => {
                            let attribute = NumberAttribute {
                                template_index: attribute_template_index,
                                value: 0,
                                perm_counter: 0,
                                node_index: current_node_index,
                            };                 

                            let attribute_index = number_attributes.len();
                            number_attributes.push(attribute);

                            node.number_attributes.push(attribute_index);
                            pending_number_attributes.push(attribute_index);

                        },
                        AttributeTemplateValue::Pos { .. } => {
                            let attribute = PosAttribute {
                                template_index: attribute_template_index,
                                value: Default::default(),
                                node_index: current_node_index,
                            };                 

                            let attribute_index = pos_attributes.len();
                            pos_attributes.push(attribute);

                            node.pos_attributes.push(attribute_index);
                            pending_pos_attributes.push(attribute_index);

                        },
                    }
                }

                debug!("Node: {}", template_node.name);
            } else {
                break;
            }
        }

        //dbg!(&nodes);
        //dbg!(&number_attributes);
        //dbg!(&pos_attributes);

        pending_nodes.push(0);

        loop {
            if let Some(current_node_index) = pending_nodes.pop() {
                let node = &mut nodes[current_node_index];
                let node_template = &self.nodes[node.template_index];
                
                for &child_index in node.children.iter() {
                    pending_nodes.push(child_index);
                }

                (node_template.build)(BuildFuncData::new(&mut nodes, &number_attributes, &pos_attributes, &self, current_node_index, build_data));


                                
            } else {
                break;
            }

        }

        //dbg!(&build_data);

    }
}

impl<U: Clone + Debug, B: Clone + Debug> WFCBuilder<U, B> {
    pub fn get_node_index_by_identifier(&self, identifier: Identifier) -> usize {
        self.nodes.iter().position(|n| n.identifier == Some(identifier)).expect("No Node with Identifier {Identifier} found.")
    }
    pub fn get_attribute_index_by_identifier(&self, identifier: Identifier) -> usize {
        self.attributes.iter().position(|n| n.identifier == identifier).expect("No Attribute with Identifier {Identifier} found.")
    }
}


