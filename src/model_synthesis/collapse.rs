use core::{panic};
use std::{collections::HashMap, fmt::Debug, marker::PhantomData, task::ready, usize};

use fdg::nalgebra::base;
use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{anyhow::{anyhow, bail}, glam::{vec3, IVec3, Vec3}, log::{debug, error, info}, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};

use crate::{vec_csg_tree::tree::VecCSGTree, model_synthesis::{volume::PossibleVolume}};

use super::builder::{NodeTemplate, NodeTemplateValue, WFCBuilder, BU, IT};

new_key_type! { pub struct CollapseNodeKey; }


#[derive(Debug, Clone)]
pub struct Collapser<'a, I: IT, U: BU> {
    builder: &'a WFCBuilder<I>,
    pub nodes: SlotMap<CollapseNodeKey, Node<I, U>>,
    pending_collapse: Vec<CollapseNodeKey>,
    pending_undo_build: Vec<(CollapseNodeKey, I)>,
}

#[derive(Debug, Clone)]
pub struct Node<I: IT, U: BU> {
    pub template_index: usize,
    pub identfier: I,
    pub children: Vec<CollapseNodeKey>,
    pub depends: Vec<(I, CollapseNodeKey)>,
    pub data: NodeDataType<U>,
    pub next_reset: CollapseNodeKey,
}

#[derive(Debug, Clone)]
pub enum NodeDataType<U: BU> {
    Number(NumberData),
    Pos(PosData),
    Volume(VolumeData),
    Build(U),
    None,
}

#[derive(Debug, Clone)]
pub struct NumberData {
    pub value: i32,
    pub perm_counter: usize,
}

#[derive(Debug, Clone)]
pub struct PosData {
    pub value: Vec3,
    pub collapsed: bool,
}

#[derive(Debug, Clone)]
pub struct VolumeData {
    pub value: PossibleVolume,
}

pub enum CollapseOperation<I> {
    None,
    CollapsePos {
        index: CollapseNodeKey,
    },
    Build {
        index: CollapseNodeKey,
        identifier: I, 
    },
    UndoBuild {
        index: CollapseNodeKey,
        identifier: I,
    }
}

impl<'a, I: IT, U: BU> Collapser<'a, I, U> {
    pub fn next(&mut self) -> Option<(CollapseOperation<I>, &mut Collapser<'a, I, U>)> {

        if let Some((index, identifier)) = self.pending_undo_build.pop() {
            return Some((CollapseOperation::UndoBuild { index, identifier }, self));
        } 

        if let Some(node_index) = self.pending_collapse.pop() {
            let node = &mut self.nodes[node_index];

            if let NodeDataType::Pos(pos_data) = &mut node.data {
                if !pos_data.collapsed {

                    pos_data.collapsed = true;
                    self.pending_collapse.push(node_index);
                    return Some((CollapseOperation::CollapsePos { 
                        index: node_index
                    }, self));
                }
            }
            

            if let NodeDataType::Build(_) = node.data {
                return Some((CollapseOperation::Build { 
                    index: node_index, 
                    identifier: node.identfier,
                }, self));
            }

            let node_template = &self.builder.nodes[node.template_index];
            if let NodeTemplateValue::NumberRange { min, max, permutation  } = &node_template.value {
                
                let node = &mut self.nodes[node_index];
                let number_value = node.data.get_number_mut();

                if number_value.perm_counter >= permutation.max() as usize {
                    info!("{:?} Resetting Number faild", node_index);

                    number_value.perm_counter = 0;
                    self.pending_collapse.push(node_index);

                    let next_reset = node.next_reset;
                    self.reset_node(next_reset); 
                    return Some((CollapseOperation::None, self));
                }

                let value = permutation.get(number_value.perm_counter as _) as i32 + *min;
                number_value.perm_counter += 1;

                number_value.value = value;
                info!("{:?} {:?}: {}", node_index, node.identfier, number_value.value);

                let collapse_len = self.pending_collapse.len();
                
                /*
                for child_identifier in node_template.children.iter() { 
                    for i in 0..value {
                        self.add_child(*child_identifier, node_index, collapse_len);
                    }
                }
                */

            } else {
                //self.add_children(node_index, node_template);
            } 
        } else {
            return None;
        }

        Some((CollapseOperation::None, self)) 
    }

    /*
    fn add_children(&mut self, node_index: CollapseNodeKey, node_template: &NodeTemplate<I>) {
        let collapse_len = self.pending_collapse.len();
        for child_identifier in node_template.children.iter() {
            self.add_child(*child_identifier, node_index, collapse_len);
        } 
    }

    fn add_child(&mut self, child_identifier: I, node_index: CollapseNodeKey, insert_collpase_at: usize) {
        let new_node_template_index = self.builder.get_node_index_by_identifier(child_identifier);
        
        let index = self.add_node(new_node_template_index, node_index, insert_collpase_at);
        self.nodes[node_index].children.push(index);
    }
    */

    pub fn add_node(&mut self, new_node_template_index: usize, parent: CollapseNodeKey, insert_collpase_at: usize) -> CollapseNodeKey {
        let new_node_template = &self.builder.nodes[new_node_template_index];

        let data = match &new_node_template.value {
            NodeTemplateValue::Groupe { .. } => {
                NodeDataType::None
            },
            NodeTemplateValue::NumberRange { .. } => {
                NodeDataType::Number(NumberData {
                    value: 0,
                    perm_counter: 0,
                })
            },
            NodeTemplateValue::Pos { .. } => {
                NodeDataType::Pos(PosData {
                    value: Vec3::ZERO,
                    collapsed: false,
                })
            },
            NodeTemplateValue::Volume { volume, .. } => {
                NodeDataType::Volume(VolumeData {
                    value: volume.to_owned(),
                })
            },
            NodeTemplateValue::BuildHook { .. } => {
                NodeDataType::Build(U::default())
            },
        };

        let depends = new_node_template.depends.iter().map(|i| {
            (*i, self.search_for_dependency(parent, *i))
        }).collect();

        let new_node = Node {
            template_index: new_node_template_index,
            identfier: new_node_template.identifier,
            children: vec![],
            depends,
            data,
            next_reset: parent,
        };

        let new_index = self.nodes.insert(new_node);

        self.pending_collapse.insert(insert_collpase_at, new_index);


        info!("{:?} Created: {:?}", new_index, new_node_template.identifier);

        new_index
    }

    pub fn search_for_dependency(&self, mut index: CollapseNodeKey, identifier: I) -> CollapseNodeKey {
        let mut last_index = CollapseNodeKey::null();
        while index != CollapseNodeKey::null() {
            let node = self.nodes.get(index).expect("In dependecy search: Parent CollapseNodeKey was not valid!");
            if node.identfier == identifier {
                return index;
            }

            let child_index = node.children.iter()
                .filter(|i| **i != last_index)
                .find(|i| {
                    let child_node = self.nodes.get(**i).expect("Child CollapseNodeKey not in Arena!");
                    child_node.identfier == identifier
                });
        
            if let Some(child_index) = child_index {
                return *child_index;
            } else {
                last_index = index;
            }
        }

        panic!("Hit root in dependecy search for {:?}", identifier);
    }
    
    pub fn get_number(&self, index: CollapseNodeKey) -> i32 {
        match &self.nodes.get(index).expect("Number by index not found").data {
            NodeDataType::Number(d) => d.value,
            _ => panic!("Number by index is not of Type Number")
        }
    }

    pub fn get_pos(&self, index: CollapseNodeKey) -> Vec3 {
        match &self.nodes.get(index).expect("Pos by index not found").data {
            NodeDataType::Pos(d) => d.value,
            _ => panic!("Pos by index is not of Type Pos")
        }
    }


    pub fn get_pos_mut(&mut self, index: CollapseNodeKey) -> &mut Vec3 {
        match &mut self.nodes.get_mut(index).expect("Pos by index not found").data {
            NodeDataType::Pos(d) => &mut d.value,
            _ => panic!("Pos by index is not of Type Pos")
        }
    }

    fn get_dependend_indecies_with_index(&self, index: CollapseNodeKey) -> &[(I, CollapseNodeKey)] {
        &self.nodes.get(index).expect("Node by index not found").depends
    }

    fn get_dependend_index(&self, index: CollapseNodeKey, identifier: I) -> CollapseNodeKey {
        let depends = self.get_dependend_indecies_with_index(index);
        depends.iter().find(|(i, _)| *i == identifier).expect(&format!("Node has no depends {:?}", identifier)).1
    }


    pub fn get_dependend_number(&self, index: CollapseNodeKey, identifier: I) -> i32 {
        let index = self.get_dependend_index(index, identifier);
        self.get_number(index)
    }

    pub fn get_dependend_pos(&self, index: CollapseNodeKey, identifier: I) -> Vec3 {
        let index = self.get_dependend_index(index, identifier);
        self.get_pos(index)
    }

    pub fn get_dependend_pos_mut(&mut self, index: CollapseNodeKey, identifier: I) -> &mut Vec3 {
        let index = self.get_dependend_index(index, identifier);
        self.get_pos_mut(index)
    }

    pub fn pos_collapse_failed(&mut self, index: CollapseNodeKey) {

    }

    pub fn build_failed(&mut self, index: CollapseNodeKey) {
        info!("{:?} Build of faild", index);
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");

        self.pending_collapse.push(index);
         
        let mut last = CollapseNodeKey::null();
        for (_, i) in node.depends.to_owned() {
            if last == CollapseNodeKey::null() {
                self.reset_node(i);
            } else {
                self.set_next_reset(last, i); 
            }

            last = i;
        }
        //self.set_next_reset(last, parent);
    }

    fn reset_node(&mut self, index: CollapseNodeKey) {
        info!("{:?} Reset Node", index);

        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        
        self.delete_children(index);

        self.pending_collapse.push(index);
    }

    fn delete_children(&mut self, index: CollapseNodeKey) {
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        
        if let NodeDataType::Build(_) = node.data {
            self.pending_undo_build.push((index, node.identfier));
        }

        for child in node.children.to_owned() {
            self.delete_children(child);
            self.nodes.remove(child);
        }
    }

    fn set_next_reset(&mut self, index: CollapseNodeKey, set_to: CollapseNodeKey) {
        let node = self.nodes.get_mut(index).expect("Reset CollapseNodeKey not valid!");
        node.next_reset = set_to;
    }


    pub fn get_build_data(&self, index: CollapseNodeKey) -> OctaResult<U> {
        let node = self.nodes.get(index)
            .ok_or(anyhow!("Index of build node to get data is not valid!"))?;
        
        if let NodeDataType::Build(d) = &node.data {
            return Ok(*d); 
        } 
        
        bail!("Node Type ({:?}) is not Build Node when trying to set Build data!", node.data);
    }

    pub fn set_build_data(&mut self, index: CollapseNodeKey, data: U) -> OctaResult<()> {
        let node = self.nodes.get_mut(index)
            .ok_or(anyhow!("Index of build node to set data is not valid!"))?;

        if let NodeDataType::Build(d) = &mut node.data {
            *d = data;
        } else {
            bail!("Node Type ({:?}) is not Build Node when trying to set Build data!", node.data);
        }

        Ok(())
    }

}


impl<U: BU> NodeDataType<U> {
    pub fn get_number_mut(&mut self) -> &mut NumberData {
        match self {
            NodeDataType::Number(d) => d,
            _ => unreachable!()
        }
    } 
}


impl<I: IT> WFCBuilder<I> {
    pub fn get_collaper<U: BU>(&self) -> Collapser<I, U> {
        let mut collapser = Collapser{
            builder: self,
            nodes: SlotMap::with_key(),
            pending_collapse: vec![],
            pending_undo_build: vec![],
        };

        collapser.add_node(0, CollapseNodeKey::null(), 0);
        collapser
    }
}


