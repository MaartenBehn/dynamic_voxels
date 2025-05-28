
use std::{collections::HashMap, fmt::{Debug, Octal}, iter, marker::PhantomData, mem, task::ready, usize};

use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, IVec3, Vec3}, log::{debug, error, info}, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};


use crate::{vec_csg_tree::tree::VecCSGTree, volume::Volume};

use super::{builder::{BuilderNode, ModelSynthesisBuilder, BU, IT}, relative_path::{LeafType, RelativePathTree}, template::{NodeTemplateValue, TemplateAmmountType, TemplateIndex, TemplateNode, TemplateTree}};

new_key_type! { pub struct CollapseNodeKey; }

#[derive(Debug, Clone)]
pub struct Collapser<'a, I: IT, U: BU> {
    pub template: &'a TemplateTree<I>,
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<I, U>>,
    pub pending_operations: Vec<NodeOperation>,
    pub pending_collapse_opperations: Vec<CollapseOperation<I, U>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub struct NodeOperation {
    pub level: usize,
    pub index: CollapseNodeKey,
    pub typ: NodeOperationType,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum NodeOperationType {
    CollapseValue,
    UpdateDefined(TemplateIndex),
}

#[derive(Debug, Clone)]
pub struct CollapseNode<I: IT, U: BU> {
    pub template_index: usize,
    pub identfier: I,
    pub level: usize,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub depends: Vec<(I, CollapseNodeKey)>,
    pub knows: Vec<(I, CollapseNodeKey)>,
    pub defined_by: CollapseNodeKey,
    pub data: NodeDataType,
    pub next_reset: CollapseNodeKey,
    pub undo_data: U,
}

#[derive(Debug, Clone)]
pub enum NodeDataType {
    Number(NumberData),
    PosSet(PosSetData),
    Pos(PosData),
    Build,
    None,
}

#[derive(Debug, Clone)]
pub struct NumberData {
    pub value: i32,
    pub perm_counter: usize,
}

#[derive(Debug, Clone)]
pub struct PosSetData {
}

#[derive(Debug, Clone)]
pub struct PosData {
    pub value: Vec3,
}

#[derive(Debug, Clone)]
pub struct GridData {

}

#[derive(Debug, Clone)]
pub enum CollapseOperation<I, U> {
    None,
    CollapsePos {
        index: CollapseNodeKey,
    },
    CollapseBuild {
        index: CollapseNodeKey,
        identifier: I, 
    },
    Undo {
        identifier: I,
        undo_data: U,
    }
}

impl<'a, I: IT, U: BU> Collapser<'a, I, U> {
    pub fn next(&mut self) -> OctaResult<Option<(CollapseOperation<I, U>, &mut Collapser<'a, I, U>)>> { 
        if let Some(collapse_opperation) = self.pending_collapse_opperations.pop() {
            Ok(Some((collapse_opperation, self)))

        } else if let Some(operation) = self.pending_operations.pop() {
            match operation.typ {
                NodeOperationType::CollapseValue => {
                    let opperation = self.collapse_node(operation.index)?;
                    Ok(Some((opperation, self)))    
                },
                NodeOperationType::UpdateDefined(defined_index) => {
                    self.update_defined(operation.index, defined_index)?;
                    Ok(Some((CollapseOperation::None, self)))
                },
            }
        } else {
            Ok(None)
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey) -> OctaResult<CollapseOperation<I, U>> {
        let node = &mut self.nodes[node_index];
        let template_node = &self.template.nodes[node.template_index]; 
        //info!("{:?} Collapse: {:?}", node_index, node.identfier);

        if let NodeDataType::Pos(pos_data) = &mut node.data {
            match template_node.value {
                NodeTemplateValue::Pos { value } => node.data.set_pos(value),
                NodeTemplateValue::PosHook => {
                    self.push_pending_defineds(node_index);
                    return Ok(CollapseOperation::CollapsePos { 
                        index: node_index
                    });
                },
                _ => unreachable!()
            }
        }

        if let NodeDataType::Build = node.data {

            return Ok(CollapseOperation::CollapseBuild { 
                index: node_index, 
                identifier: node.identfier,
            });
        }

        let node_template = &self.template.nodes[node.template_index];
        if let NodeTemplateValue::NumberRange { min, max, permutation  } = &node_template.value {

            let node = &mut self.nodes[node_index];
            let number_value = node.data.get_number_mut();

            if number_value.perm_counter >= permutation.max() as usize {
                info!("{:?} Resetting Number faild", node_index);

                number_value.perm_counter = 0;
                
                let next_reset = node.next_reset;
                self.reset_node(next_reset)?;

                self.insert_opperation(NodeOperation {
                    index: node_index,
                    level: node_template.level,
                    typ: NodeOperationType::CollapseValue,
                });

                return Ok(CollapseOperation::None);
            }

            let value = permutation.get(number_value.perm_counter as _) as i32 + min;
            number_value.perm_counter += 1;

            number_value.value = value;
            info!("{:?} {:?}: {}", node_index, node.identfier, number_value.value); 
        }
 
        self.push_pending_defineds(node_index);
        Ok(CollapseOperation::None)
    }

    fn push_pending_defineds(&mut self, node_index: CollapseNodeKey) {
        let node = &self.nodes[node_index];
        let template_node = &self.template.nodes[node.template_index]; 
        for ammount in template_node.defines_ammount.iter() {
            let new_node_template = &self.template.nodes[ammount.index];

            self.insert_opperation(NodeOperation{
                level: new_node_template.level,
                index: node_index,
                typ: NodeOperationType::UpdateDefined(ammount.index),
            }); 
        }
    }

    pub fn pos_collapse_failed(&mut self, index: CollapseNodeKey) {

    }

    pub fn collapse_failed(&mut self, index: CollapseNodeKey) -> OctaResult<()> {        
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        info!("{:?} Collapse Faild {:?}", index, node.identfier);

        let level = self.template.nodes[node.template_index].level;
        
        Self::insert_opperation_unpacked(&mut self.pending_operations, NodeOperation{
            level,
            index,
            typ: NodeOperationType::CollapseValue,
        });

        let mut last = CollapseNodeKey::null();
        for (_, i) in node.depends.to_owned().into_iter().rev() {
            if last.is_null() {
                self.reset_node(i)?;
            } else {
                self.set_next_reset(last, i)?; 
            }

            last = i;
        }

        Ok(())
    }

     
    pub fn set_undo_data(&mut self, index: CollapseNodeKey, data: U) -> OctaResult<()> {
        let node = self.nodes.get_mut(index)
            .ok_or(anyhow!("Index of build node to set data is not valid!"))?;
        
        node.undo_data = data;
        
        Ok(())
    }

}


impl NodeDataType {
    pub fn get_number_mut(&mut self) -> &mut NumberData {
        match self {
            NodeDataType::Number(d) => d,
            _ => unreachable!()
        }
    } 

    pub fn get_pos_mut(&mut self) -> &mut PosData {
        match self {
            NodeDataType::Pos(d) => d,
            _ => unreachable!()
        }
    }

    pub fn set_pos(&mut self, v: Vec3) {
        match self {
            NodeDataType::Pos(d) => d.value = v,
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
            pending_collapse_opperations: vec![],
        };

        collapser.add_node(0, vec![], vec![], CollapseNodeKey::null());
        collapser
    }
}

impl Ord for NodeOperation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.level.cmp(&self.level)
            .then(other.typ.cmp(&self.typ))
            .then(other.index.cmp(&self.index))
    }
}
