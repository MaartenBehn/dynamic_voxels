
use std::{collections::HashMap, fmt::{Debug, Octal}, iter, marker::PhantomData, mem, task::ready, usize};

use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, IVec3, Vec3}, log::{debug, error, info}, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};


use crate::{model::generation::pending_operations::NodeOperation, volume::VolumeQureyPosValid};

use super::{builder::{BuilderNode, ModelSynthesisBuilder, BU, IT}, pending_operations::PendingOperations, pos_set::PositionSet, relative_path::{LeafType, RelativePathTree}, template::{NodeTemplateValue, TemplateAmmountType, TemplateIndex, TemplateNode, TemplateTree}};

new_key_type! { pub struct CollapseNodeKey; }

#[derive(Debug, Clone)]
pub struct Collapser<I: IT, U: BU, V: VolumeQureyPosValid> {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<I, U, V>>,
    pub pending_operations: PendingOperations,
    pub pending_collapse_opperations: Vec<CollapseOperation<I, U>>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum NodeOperationType {
    CollapseValue,
    UpdateDefined(TemplateIndex),
}

#[derive(Debug, Clone)]
pub struct CollapseNode<I: IT, U: BU, V: VolumeQureyPosValid> {
    pub template_index: usize,
    pub identfier: I,
    pub level: usize,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub depends: Vec<(I, CollapseNodeKey)>,
    pub knows: Vec<(I, CollapseNodeKey)>,
    pub defined_by: CollapseNodeKey,
    pub data: NodeDataType<V>,
    pub next_reset: CollapseNodeKey,
    pub undo_data: U,
}

#[derive(Debug, Clone)]
pub enum NodeDataType<V: VolumeQureyPosValid> {
    Number(NumberData),
    PosSet(PosSetData<V>),
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
pub struct PosSetData<V: VolumeQureyPosValid> {
    pub set: PositionSet<V>
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
    NumberRangeHook {
        index: CollapseNodeKey,
    },
    PosSetHook {
        index: CollapseNodeKey,
    },
    PosHook {
        index: CollapseNodeKey,
    },
    BuildHook {
        index: CollapseNodeKey,
        identifier: I, 
    },
    Undo {
        identifier: I,
        undo_data: U,
    }
}

impl<I: IT, U: BU, V: VolumeQureyPosValid> Collapser<I, U, V> {
    pub fn next(&mut self, template: &TemplateTree<I, V>) 
    -> OctaResult<Option<(CollapseOperation<I, U>, &mut Collapser<I, U, V>)>> {

        if let Some(collapse_opperation) = self.pending_collapse_opperations.pop() {
            return Ok(Some((collapse_opperation, self)));
        }

        if let Some(operation) = self.pending_operations.pop() {
            match operation.typ {
                NodeOperationType::CollapseValue => {
                    let opperation = self.collapse_node(operation.key, template)?;
                    Ok(Some((opperation, self)))    
                },
                NodeOperationType::UpdateDefined(defined_index) => {
                    self.update_defined(operation.key, defined_index, template)?;
                    Ok(Some((CollapseOperation::None, self)))
                },
            }
        } else {
            Ok(None)
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<I, V>) -> OctaResult<CollapseOperation<I, U>> {
        let node = &mut self.nodes[node_index];
        let template_node = &template.nodes[node.template_index]; 
        //info!("{:?} Collapse: {:?}", node_index, node.identfier);

        match &mut node.data {
            NodeDataType::Number(number_data) => {
                match &template_node.value {
                    NodeTemplateValue::NumberRangeHook => {
                        self.push_pending_defineds(node_index, template);
                        return Ok(CollapseOperation::NumberRangeHook { 
                            index: node_index
                        });
                    },
                    NodeTemplateValue::NumberRange { min, max, permutation } => {

                        if number_data.perm_counter >= permutation.max() as usize {
                            info!("{:?} Resetting Number faild", node_index);

                            number_data.perm_counter = 0;

                            let next_reset = node.next_reset;
                            self.reset_node(next_reset, template)?;

                            self.pending_operations.push(template_node.level, NodeOperation { 
                                key: node_index, 
                                typ: NodeOperationType::CollapseValue,
                            });

                            return Ok(CollapseOperation::None);
                        }

                        let value = permutation.get(number_data.perm_counter as _) as i32 + min;
                        number_data.perm_counter += 1;

                        number_data.value = value;
                        info!("{:?} {:?}: {}", node_index, node.identfier, number_data.value); 
                    },
                    _ => unreachable!()
                }
            },
            NodeDataType::PosSet(pos_set_data) => {
                match &template_node.value {
                    NodeTemplateValue::PosSetHook => {
                        self.push_pending_defineds(node_index, template);
                        return Ok(CollapseOperation::PosHook { 
                            index: node_index
                        });
                    },
                    NodeTemplateValue::PosSet(position_set) => {
                        pos_set_data.set = position_set.clone() 
                    },
                    _ => unreachable!(),
                }
            },
            NodeDataType::Pos(pos_data) => {
                match template_node.value {
                    NodeTemplateValue::Pos { value } => node.data.set_pos(value),
                    NodeTemplateValue::PosHook => {
                        self.push_pending_defineds(node_index, template);
                        return Ok(CollapseOperation::PosHook { 
                            index: node_index
                        });
                    },
                    _ => unreachable!()
                }
            },
            NodeDataType::Build => {
                return Ok(CollapseOperation::BuildHook { 
                    index: node_index, 
                    identifier: node.identfier,
                });
            },
            NodeDataType::None => {},
        }

        self.push_pending_defineds(node_index, template);
        Ok(CollapseOperation::None)
    }

    fn push_pending_defineds(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<I, V>) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for ammount in template_node.defines_ammount.iter() {
            let new_node_template = &template.nodes[ammount.index];

            self.pending_operations.push(new_node_template.level, NodeOperation { 
                key: node_index, 
                typ: NodeOperationType::UpdateDefined(ammount.index),
            });
        }
    }

    pub fn pos_collapse_failed(&mut self, index: CollapseNodeKey) {
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        info!("{:?} Pos Collapse Faild {:?}", index, node.identfier);
        todo!();
    }

    pub fn collapse_failed(&mut self, index: CollapseNodeKey, template: &TemplateTree<I, V>) -> OctaResult<()> {        
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        info!("{:?} Collapse Faild {:?}", index, node.identfier);

        let level = template.nodes[node.template_index].level;

        self.pending_operations.push(level, NodeOperation {
            key: index,
            typ: NodeOperationType::CollapseValue,
        });

        let mut last = CollapseNodeKey::null();
        for (_, i) in node.depends.to_owned().into_iter().rev() {
            if last.is_null() {
                self.reset_node(i, template)?;
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


impl<V: VolumeQureyPosValid> NodeDataType<V> {
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



impl<I: IT, V: VolumeQureyPosValid> TemplateTree<I, V> {
    pub fn get_collaper<U: BU>(&self) -> Collapser<I, U, V> {
        let inital_capacity = 1000;

        let mut collapser = Collapser{
            nodes: SlotMap::with_capacity_and_key(inital_capacity),
            pending_operations: PendingOperations::new(self.max_level),
            pending_collapse_opperations: Vec::with_capacity(inital_capacity),
        };

        collapser.add_node(0, vec![], vec![], CollapseNodeKey::null(), self);
        collapser
    }
}
