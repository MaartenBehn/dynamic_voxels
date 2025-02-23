use core::panic;
use std::fmt::Debug;

use fdg::nalgebra::base;
use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{glam::{vec3, IVec3, Vec3}, log::debug};

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
    pub value: AttributeValue,
    pub perm_counter: usize,
}

#[derive(Debug, Clone)]
pub struct Node<U: Clone + Debug> {
    pub template_index: usize,
    pub attributes: Vec<usize>,
    pub user_data: Option<U>
}

#[derive(Debug, Clone)]
pub struct CollapseFuncData<'a, U: Clone + Debug> {
    pub nodes: &'a[Node<U>],
    pub attributes: &'a[Attribute],
    pub builder: &'a WFCBuilder<U>,
    pub value: usize,
}


impl<U: Clone + Debug> WFCBuilder<U> {
    pub fn collapse(self) {

        let root_node_template = &self.nodes[0];
        let root_node = Node { 
            attributes: vec![], 
            template_index: 0,
            user_data: root_node_template.user_data.to_owned(),
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
                                };

                                let node_index = nodes.len();
                                nodes.push(node);
                                pending_nodes.push(node_index);
                            }
                        }
                    },
                    AttributeTemplateValue::Pos { collapse, ..  } => {
                        let pos = collapse(CollapseFuncData{
                            nodes: &nodes, 
                            attributes: &attributes, 
                            builder: &self,
                            value: value as _,
                        });
                        
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

        dbg!(nodes);
        dbg!(attributes);
    }
}

impl<U: Clone + Debug> WFCBuilder<U> {
    pub fn get_node_index_by_identifier(&self, identifier: Identifier) -> usize {
        self.nodes.iter().position(|n| n.identifier == Some(identifier)).expect("No Node with Identifier {Identifier} found.")
    }
    pub fn get_attribute_index_by_identifier(&self, identifier: Identifier) -> usize {
        self.attributes.iter().position(|n| n.identifier == identifier).expect("No Attribute with Identifier {Identifier} found.")
    }
}

impl<'a, U: Clone + Debug> CollapseFuncData<'a, U> {
    pub fn get_node_with_identifier(&self, identifer: Identifier) -> Option<&Node<U>> {
        for node in self.nodes.iter().rev() {
            let template_node = &self.builder.nodes[node.template_index];
            if template_node.identifier == Some(identifer) {
                return Some(node);
            }
        }

        None
    }

    pub fn get_attribute_with_identifier(&self, identifer: Identifier, mut skip: usize) -> Option<&Attribute> {
        for attribute in self.attributes.iter().rev() {
            let template_attribute = &self.builder.attributes[attribute.template_index];
            if template_attribute.identifier == identifer {
                if skip > 0 {
                    skip -= 1;
                } else {
                    return Some(attribute);
                }
            }
        }

        None
    }

}
