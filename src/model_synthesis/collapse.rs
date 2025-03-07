use core::panic;
use std::{fmt::Debug, marker::PhantomData, task::ready, usize};

use fdg::nalgebra::base;
use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{glam::{vec3, IVec3, Vec3}, log::{debug, error}};
use thunderdome::{Arena, Index};

use crate::{csg_tree::tree::CSGTree, model_synthesis::{volume::PossibleVolume}};

use super::{builder::{AttributeTemplate, AttributeTemplateValue, NodeTemplate, NumberRangeDefines, WFCBuilder, IT}};

#[derive(Debug, Clone)]
pub struct Collapser<'a, I: IT> {
    builder: &'a WFCBuilder<I>,
    pub nodes: Arena<Node>,
    pending_nodes: Vec<Index>,

    pub number_attributes: Arena<NumberAttribute<I>>,
    pending_numbers: Vec<Index>,

    pub pos_attributes: Arena<PosAttribute<I>>,
    pending_pos: Vec<Index>,

    pub volume_attributes: Arena<VolumeAttribute<I>>,

    pending_build: Vec<(Index, I)>,
}


#[derive(Debug, Clone)]
pub struct Node {
    pub template_index: usize,
    pub number_attributes: Vec<Index>,
    pub pos_attributes: Vec<Index>,
    pub volume_attributes: Vec<Index>,
    pub children_nodes: Vec<Index>,
}

#[derive(Debug, Clone)]
pub struct NumberAttribute<I> {
    pub template_index: usize,
    pub node_index: Index,
    pub value: i32,
    pub perm_counter: usize,
    pub identfier: I,
}

#[derive(Debug, Clone)]
pub struct PosAttribute<I> {
    pub template_index: usize,
    pub node_index: Index,
    pub value: Vec3,
    pub identfier: I,
}

#[derive(Debug, Clone)]
pub struct VolumeAttribute<I> {
    pub template_index: usize,
    pub node_index: Index,
    pub value: PossibleVolume,
    pub identfier: I,
}

pub enum CollapseOperation<I> {
    None,
    CollapsePos {
        index: Index,
    },
    BuildNode {
        index: Index,
        identifier: I, 
    },
}

impl<'a, I: IT> Collapser<'a, I> {
    pub fn next(&mut self) -> Option<(CollapseOperation<I>, &mut Collapser<'a, I>)> {

        if let Some(current_attribute_index) = self.pending_numbers.pop() {
            let attribute = self.number_attributes.get_mut(current_attribute_index).expect("Pending Number Attribute was not in Arena!");
            let attribute_template = &self.builder.attributes[attribute.template_index];
            
            if attribute.perm_counter >= attribute_template.value.get_number_permutation().max() as usize {
                error!("Number collapse failed"); 
                return None;
            }

            let value = attribute_template.value.get_number_permutation().get(attribute.perm_counter as _);
            attribute.perm_counter += 1;

            attribute.value = value as i32 + attribute_template.value.get_number_min();
            debug!("{:?}: {}", attribute.identfier, attribute.value);

            if let NumberRangeDefines::Amount { of_node, .. } = attribute_template.value.get_number_defines() {
                 
                for i in 0..attribute.value {
                    let node_template_index = self.builder.get_node_index_by_identifier(*of_node);
                    let node_template = &self.builder.nodes[node_template_index];

                    let node_index = self.nodes.insert(Node {
                        template_index: node_template_index,
                        number_attributes: vec![], 
                        pos_attributes: vec![],
                        volume_attributes: vec![],
                        children_nodes: vec![],
                    });

                    self.pending_nodes.push(node_index);

                    if node_template.build_hook {
                        self.pending_build.push((node_index, *of_node));
                    }

                    self.nodes.get_mut(attribute.node_index)
                        .expect("Attribute Parent not in Arena!")
                        .children_nodes
                        .push(node_index);
                }
            }
                
        } else if let Some(current_attribute_index) = self.pending_pos.pop() {
            let attribute = self.pos_attributes.get_mut(current_attribute_index).expect("Pending Pos Attribute was not in Arena!");
            let attribute_template = &self.builder.attributes[attribute.template_index];
                        
            return Some((CollapseOperation::CollapsePos { 
                index: current_attribute_index
            }, self));

        } else if let Some(current_node_index) = self.pending_nodes.pop() {
            let node = self.nodes.get_mut(current_node_index).expect("Pending Node was not in Arena!");
            let template_node = &self.builder.nodes[node.template_index];

            for attribute_template_identifier in template_node.attributes.iter() {
                let attribute_template_index = self.builder.get_attribute_index_by_identifier(*attribute_template_identifier);
                let attribute_template = &self.builder.attributes[attribute_template_index];

                let attribute_index = match &attribute_template.value {
                    AttributeTemplateValue::NumberRange { .. } => {
                        let attribute_index = self.number_attributes.insert(NumberAttribute {
                            template_index: attribute_template_index,
                            value: 0,
                            perm_counter: 0,
                            node_index: current_node_index,
                            identfier: *attribute_template_identifier,
                        });

                        node.number_attributes.push(attribute_index);
                        self.pending_numbers.push(attribute_index);

                        attribute_index
                    },
                    AttributeTemplateValue::Pos { .. } => {
                        let attribute_index = self.pos_attributes.insert(PosAttribute {
                            template_index: attribute_template_index,
                            value: Default::default(),
                            node_index: current_node_index,
                            identfier: *attribute_template_identifier,
                        });                 
                        
                        node.pos_attributes.push(attribute_index);
                        self.pending_pos.push(attribute_index);

                        attribute_index
                    },
                    AttributeTemplateValue::Volume { volume, .. } => {
                        let attribute_index = self.volume_attributes.insert(VolumeAttribute {
                            template_index: attribute_template_index,
                            node_index: current_node_index,
                            value: volume.clone(),
                            identfier: *attribute_template_identifier,
                        });

                        node.volume_attributes.push(attribute_index);

                        attribute_index
                    },
                };

                if attribute_template.build_hook {
                    self.pending_build.push((attribute_index, attribute_template.identifier));
                }

            }

            for child in template_node.children.iter() {
                let node_template_index = self.builder.get_node_index_by_identifier(*child);
                let node_template = &self.builder.nodes[node_template_index];

                let node_index = self.nodes.insert(Node {
                    template_index: node_template_index,
                    number_attributes: vec![], 
                    pos_attributes: vec![],
                    volume_attributes: vec![],
                    children_nodes: vec![],
                });

                self.pending_nodes.push(node_index);

                if node_template.build_hook {
                    self.pending_build.push((node_index, *child));
                }

                self.nodes.get_mut(current_node_index)
                    .expect("Pending Node was not in Arena!")
                    .children_nodes
                    .push(node_index);
            } 

        } else if let Some((index, identifier)) = self.pending_build.pop() {
            return Some((CollapseOperation::BuildNode { 
                index, 
                identifier,
            }, self));


        } else {
            return None;
        }

        Some((CollapseOperation::None, self)) 
    }

    pub fn get_number_with_identifier(&self, identifier: I) -> i32 {
        self.number_attributes.iter()
            .find(|(_, n)| n.identfier == identifier)
            .expect(&format!("Did not find Number Attribute {:?}", identifier))
            .1.value
    }
}



impl<I: IT> WFCBuilder<I> {
    pub fn get_collaper(&self) -> Collapser<I> {
        let mut nodes = Arena::new();
        let root_index = nodes.insert(Node {
            template_index: 0,
            number_attributes: vec![],
            pos_attributes: vec![],
            volume_attributes: vec![],
            children_nodes: vec![],
        });

        let mut pending_build = vec![];
        let root_node = &self.nodes[0];
        if root_node.build_hook {
            pending_build.push((root_index, root_node.identifier));
        }

        Collapser{
            builder: self,
            nodes,
            number_attributes: Arena::new(),
            pos_attributes: Arena::new(),
            volume_attributes: Arena::new(),
            pending_nodes: vec![root_index],
            pending_numbers: vec![],
            pending_pos: vec![],
            pending_build,
        }
    }
}

impl<I: IT> WFCBuilder<I> {
    pub fn get_node_index_by_identifier(&self, identifier: I) -> usize {
        self.nodes.iter().position(|n| n.identifier == identifier).expect("No Node with Identifier {Identifier} found.")
    }
    pub fn get_attribute_index_by_identifier(&self, identifier: I) -> usize {
        self.attributes.iter().position(|n| n.identifier == identifier).expect("No Attribute with Identifier {Identifier} found.")
    }
}


