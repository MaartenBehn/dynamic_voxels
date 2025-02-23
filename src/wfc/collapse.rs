use core::panic;
use std::fmt::Debug;

use fdg::nalgebra::base;
use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{glam::{vec3, IVec3, Vec3}, log::debug};

use crate::wfc::func_data::{BuildFuncData, CollapseFuncData};

use super::builder::{AttributeTemplate, AttributeTemplateValue, Identifier, NodeTemplate, NumberRangeDefinesType, WFCBuilder};

#[derive(Debug, Clone)]
pub enum AttributeValue {
    None,
    Number(i32),
    Pos(Vec3),
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub template_index: usize,
    pub node_index: usize,
    pub value: AttributeValue,
    pub perm_counter: usize,
}

#[derive(Debug, Clone)]
pub struct Node<U: Clone + Debug> {
    pub template_index: usize,
    pub attributes: Vec<usize>,
    pub user_data: Option<U>,
    pub children: Vec<usize>,
}



impl<U: Clone + Debug, B: Clone + Debug> WFCBuilder<U, B> {
    pub fn collapse(self, build_data: &mut B) {

        let root_node_template = &self.nodes[0];
        let root_node = Node { 
            attributes: vec![], 
            template_index: 0,
            user_data: root_node_template.user_data.to_owned(),
            children: vec![],
        };

        let mut nodes = vec![root_node];
        let mut attributes = vec![];
        let mut pending_nodes = vec![0];
        let mut pending_attributes = vec![];

        let mut current_template_node = 0;
        loop {
            if let Some(current_attribute_index) = pending_attributes.pop() {
                let attribute: &mut Attribute = &mut attributes[current_attribute_index];
                let attribute_template = &self.attributes[attribute.template_index];
                
                if attribute.perm_counter >= attribute_template.permutation.max() as _ {
                    
                }

                let value = attribute_template.permutation.get(attribute.perm_counter as _);
                attribute.perm_counter += 1;



                debug!("Value: {value}");


                match &attribute_template.value {
                    AttributeTemplateValue::NumberRange { defines, min, .. } => {
                        
                        attribute.value = AttributeValue::Number(value as i32 + min);


                        if let NumberRangeDefinesType::Amount { of_node, .. } = defines {

                            let ammount = if let AttributeValue::Number(ammount) = attribute.value {
                                ammount
                            } else { unreachable!() };

                            for i in 0..ammount {
                                let node_template_index = self.get_node_index_by_identifier(*of_node);
                                let node_template = &self.nodes[node_template_index];

                                let node = Node {
                                    template_index: node_template_index,
                                    attributes: vec![],
                                    user_data: node_template.user_data.to_owned(),
                                    children: vec![],
                                };

                                let node_index = nodes.len();
                                nodes.push(node);
                                pending_nodes.push(node_index);

                                nodes[attribute.node_index].children.push(node_index);
                            }
                        }
                    },
                    AttributeTemplateValue::Pos { collapse, ..  } => {
                        let pos = collapse(CollapseFuncData::new(&mut nodes, &attributes, &self, value as _));
                        
                        if pos.is_none() {
                            debug!("Pos is none");

                            pending_attributes.push(current_attribute_index);
                            continue;
                        }

                        debug!("Pos: {pos:?}");
                        let attribute: &mut Attribute = &mut attributes[current_attribute_index];
                        attribute.value = AttributeValue::Pos(pos.unwrap());
                    },
                }
            } else if let Some(current_node_index) = pending_nodes.pop() {
                let node = &mut nodes[current_node_index];            
                let template_node = &self.nodes[node.template_index];
                
                for attribute_template_identifier in template_node.attributes.iter() {
                    let attribute_template_index = self.get_attribute_index_by_identifier(*attribute_template_identifier);
                    let attribute_template = &self.attributes[attribute_template_index];

                    let attribute = Attribute {
                        template_index: attribute_template_index,
                        value: AttributeValue::None,
                        perm_counter: 0,
                        node_index: current_node_index,
                    };                 

                    let attribute_index = attributes.len();
                    attributes.push(attribute);

                    node.attributes.push(attribute_index);
                    pending_attributes.push(attribute_index);
                }

                debug!("Node: {}", template_node.name);
            } else {
                break;
            }
        }

        dbg!(&nodes);
        dbg!(&attributes);

        pending_nodes.push(0);

        loop {
            if let Some(current_node_index) = pending_nodes.pop() {
                let node = &mut nodes[current_node_index];
                let node_template = &self.nodes[node.template_index];
                
                for &child_index in node.children.iter() {
                    pending_nodes.push(child_index);
                }

                (node_template.build)(BuildFuncData::new(&mut nodes, &attributes, &self, current_node_index, build_data));


                                
            } else {
                break;
            }

        }

        dbg!(&build_data);

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


