
use core::panic;
use std::{collections::HashMap, fmt::Debug, iter, marker::PhantomData, task::ready, usize};

use fdg::nalgebra::base;
use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{anyhow::{anyhow, bail}, glam::{vec3, IVec3, Vec3}, log::{debug, error, info}, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};

use crate::{vec_csg_tree::tree::VecCSGTree, model_synthesis::volume::PossibleVolume};

use super::{builder::{BuilderNode, ModelSynthesisBuilder, NodeTemplateValue, BU, IT}, relative_path::RelativePathTree, template::{TemplateAmmountType, TemplateIndex, TemplateNode, TemplateTree}};

new_key_type! { pub struct CollapseNodeKey; }

#[derive(Debug, Clone)]
pub struct Collapser<'a, I: IT, U: BU> {
    template: &'a TemplateTree<I>,
    pub nodes: SlotMap<CollapseNodeKey, Node<I, U>>,
    pending_operations: Vec<NodeOperation>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct NodeOperation {
    pub level: usize,
    pub index: CollapseNodeKey,
    pub typ: NodeOperationType,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum NodeOperationType {
    CollapseValue,
    CreateDefined(usize),
}

#[derive(Debug, Clone)]
pub struct Node<I: IT, U: BU> {
    pub template_index: usize,
    pub identfier: I,
    pub children: Vec<CollapseNodeKey>, 
    pub depends: Vec<(I, CollapseNodeKey)>,
    pub data: NodeDataType<U>,
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
        if let Some(operation) = self.pending_operations.pop() {
            match operation.typ {
                NodeOperationType::CollapseValue => {
                    let opperation = self.collapse_node(operation.index);
                    Some((opperation, self))    
                },
                NodeOperationType::CreateDefined(defined_index) => {
                    self.create_defined(operation.index, defined_index);
                    Some((CollapseOperation::None, self))
                },
            }
        } else {
            None
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey) -> CollapseOperation<I> {
        let node = &mut self.nodes[node_index];
        info!("Collapse: {:?}", node.identfier);

        if let NodeDataType::Pos(pos_data) = &mut node.data {
            if !pos_data.collapsed {
                
                self.push_pending_defineds(node_index);
                return CollapseOperation::CollapsePos { 
                    index: node_index
                };
            }
        }

        if let NodeDataType::Build(_) = node.data {

            return CollapseOperation::Build { 
                index: node_index, 
                identifier: node.identfier,
            };
        }

        let node_template = &self.template.nodes[node.template_index];
        if let NodeTemplateValue::NumberRange { min, max, permutation  } = &node_template.value {

            let node = &mut self.nodes[node_index];
            let number_value = node.data.get_number_mut();

            if number_value.perm_counter >= permutation.max() as usize {
                info!("{:?} Resetting Number faild", node_index);

                /*
                number_value.perm_counter = 0;
                self.pending_operations.push(node_index);

                let next_reset = node.next_reset;
                self.reset_node(next_reset); 
                */
                return CollapseOperation::None;
            }

            let value = permutation.get(number_value.perm_counter as _) as i32 + *min;
            number_value.perm_counter += 1;

            number_value.value = value;
            info!("{:?} {:?}: {}", node_index, node.identfier, number_value.value); 
        }

        self.push_pending_defineds(node_index);
        CollapseOperation::None
    }

    fn push_pending_defineds(&mut self, node_index: CollapseNodeKey) {
        let node = &self.nodes[node_index];
        let template_node = &self.template.nodes[node.template_index]; 
        for (i, ammount) in template_node.defines_ammount.iter().enumerate() {
            let new_node_template = &self.template.nodes[ammount.index];

            self.insert_opperation(NodeOperation{
                level: new_node_template.level,
                index: node_index,
                typ: NodeOperationType::CreateDefined(i),
            }); 
        }
    }

    fn create_defined(&mut self, node_index: CollapseNodeKey, definde_index: usize) {
        let template_node = &self.template.nodes[self.nodes[node_index].template_index];
        let template_ammount = &template_node.defines_ammount[definde_index];
        let new_node_template = &self.template.nodes[template_ammount.index];
        let tree = &template_ammount.dependecy_tree;

        // Contains a list of node indecies matching the template dependency
        let mut dependencies = iter::repeat_with(|| vec![])
            .take(new_node_template.depends.len())
            .collect::<Vec<_>>();

        let mut pending_paths = tree.starts.iter()
            .map(|start| {
                (&tree.steps[*start], node_index)
            }).collect::<Vec<_>>();

        while let Some((step, index)) = pending_paths.pop() {
            let step_node = &self.nodes[index];
             
            let edges = if step.up { 
                step_node.depends.iter()
                    .map(|(_, i)|*i)
                    .filter(|i| self.nodes[*i].template_index == step.into_index)
                    .collect::<Vec<_>>()
            } else { 
                step_node.children.iter()
                    .map(|i|*i)
                    .filter(|i| self.nodes[*i].template_index == step.into_index)
                    .collect::<Vec<_>>()
            };

            if step.leaf {
                // TODO maybe precompute the index to the dependency in the relative tree
                let i = new_node_template.depends.iter()
                        .position(|i| step.into_index == *i)
                        .expect("Leaf Node in realtive Tree is not dependency");

                for edge in edges.iter() {
                    dependencies[i].push(*edge);
                }
            }

            for edge in edges {
                for child_index in step.children.iter() {
                    let child_step = &tree.steps[*child_index];
                    pending_paths.push((child_step, edge))
                }
            }  
        }

        let dependencies = new_node_template.depends.iter()
            .zip(dependencies)
            .map(|(depend_template_node, nodes)| {
                if *depend_template_node == template_node.index {
                    return (template_node.identifier, node_index);
                }

                let depend_template_node = &self.template.nodes[*depend_template_node];
                assert_eq!(nodes.len(), 1, "Invalid number of nodes for dependency of node found");
                (depend_template_node.identifier, nodes[0])
            }).collect::<Vec<_>>();

        let ammount = match template_ammount.typ {
            TemplateAmmountType::N(n) => n,
            TemplateAmmountType::Value => {
                if let NodeDataType::Number(data) = &self.nodes[node_index].data {
                    data.value as usize
                } else {
                    panic!("TemplateAmmount Value is not allowed on {:?}", &self.nodes[node_index].data);
                }
            },
        };

        for _ in 0..ammount {
            self.add_node(template_ammount.index, dependencies.clone()); 
        } 
    }
    
    fn insert_opperation(&mut self, opperation: NodeOperation) {
        let res = self.pending_operations.binary_search(&opperation);
        if let Err(index) = res {
            self.pending_operations.insert(index, opperation);
        } 
    } 
 
    pub fn add_node(&mut self, new_node_template_index: TemplateIndex, depends: Vec<(I, CollapseNodeKey)>) {
        let new_node_template = &self.template.nodes[new_node_template_index];

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

        let index = self.nodes.insert(Node {
            template_index: new_node_template_index,
            identfier: new_node_template.identifier,
            children: vec![],
            depends: depends.clone(),
            data,
        });

        for (_, depend) in depends {
            self.nodes[depend].children.push(index);
        }

        self.insert_opperation(NodeOperation {
            level: new_node_template.level,
            index,
            typ: NodeOperationType::CollapseValue,
        });
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
        dbg!(&depends);
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
        /*
        info!("{:?} Build of faild", index);
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");

        self.pending_operations.push(index);

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
        */
    }

    fn reset_node(&mut self, index: CollapseNodeKey) {
        /*
        info!("{:?} Reset Node", index);

        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");

        self.delete_children(index);

        self.pending_operations.push(index);
        */
    }

    fn delete_children(&mut self, index: CollapseNodeKey) {
        /*
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");

        if let NodeDataType::Build(_) = node.data {
            self.pending_undo_build.push((index, node.identfier));
        }

        for child in node.children.to_owned() {
            self.delete_children(child);
            self.nodes.remove(child);
        }
        */
    }

    fn set_next_reset(&mut self, index: CollapseNodeKey, set_to: CollapseNodeKey) {
        /*
        let node = self.nodes.get_mut(index).expect("Reset CollapseNodeKey not valid!");
        node.next_reset = set_to;
        */
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


impl<I: IT> TemplateTree<I> {
    pub fn get_collaper<U: BU>(&self) -> Collapser<I, U> {
        let mut collapser = Collapser{
            template: self,
            nodes: SlotMap::with_key(),
            pending_operations: vec![],
        };

        collapser.add_node(0, vec![]);
        collapser
    }
}

impl Ord for NodeOperation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.level.cmp(&self.level).then(other.typ.cmp(&self.typ))
    }
}
