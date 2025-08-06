
use std::{collections::HashMap, fmt::{Debug, Octal}, iter, marker::PhantomData, mem, task::ready, usize};

use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, IVec3, Vec3}, log::{debug, error, info}, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};


use crate::{model::generation::pos_set::PositionSetRule, volume::{VolumeQureyPosValid, VolumeQureyPosValid2D}};

use super::{builder::{BuilderNode, ModelSynthesisBuilder}, number_range::NumberRange, pending_operations::PendingOperations, pos_set::PositionSet, relative_path::{LeafType, RelativePathTree}, template::{NodeTemplateValue, TemplateIndex, TemplateNode, TemplateTree}, traits::ModelGenerationTypes};

new_key_type! { pub struct CollapseNodeKey; }
new_key_type! { pub struct CollapseChildKey; }

#[derive(Debug, Clone)]
pub struct Collapser<T: ModelGenerationTypes> {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<T>>,
    pub pending_collapses: PendingOperations,
    pub pending_collapse_opperations: Vec<CollapseOperation<T>>,
}

#[derive(Debug, Clone)]
pub struct CollapseNode<T: ModelGenerationTypes> {
    pub template_index: usize,
    pub identifier: T::Identifier,
    pub level: usize,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub depends: Vec<(T::Identifier, CollapseNodeKey)>,
    pub knows: Vec<(T::Identifier, CollapseNodeKey)>,
    pub defined_by: CollapseNodeKey,
    pub child_key: CollapseChildKey,
    pub data: NodeDataType<T>,
    pub next_reset: CollapseNodeKey,
    pub undo_data: T::UndoData,
}

#[derive(Debug, Clone)]
pub enum NodeDataType<T: ModelGenerationTypes> {
    NumberRange(NumberRange),
    PosSet(PositionSet<T>),
    Build,
    None,
    NotValid
}

#[derive(Debug, Clone)]
pub enum CollapseOperation<T: ModelGenerationTypes> {
    None,
    NumberRangeHook {
        index: CollapseNodeKey,
        identifier: T::Identifier, 
    },
    PosSetHook {
        index: CollapseNodeKey,
        identifier: T::Identifier, 
    },
    BuildHook {
        index: CollapseNodeKey,
        identifier: T::Identifier, 
    },
    Undo {
        identifier: T::Identifier,
        undo_data: T::UndoData,
    }
}

impl<T: ModelGenerationTypes> Collapser<T> {
    pub fn next(&mut self, template: &TemplateTree<T>) 
    -> OctaResult<Option<(CollapseOperation<T>, &mut Collapser<T>)>> {

        if let Some(collapse_opperation) = self.pending_collapse_opperations.pop() {
            return Ok(Some((collapse_opperation, self)));
        }

        if let Some(key) = self.pending_collapses.pop() {
            let opperation = self.collapse_node(key, template)?;
            Ok(Some((opperation, self)))
        } else {
            Ok(None)
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>) -> OctaResult<CollapseOperation<T>> {
        let node = &mut self.nodes[node_index];
        let template_node = &template.nodes[node.template_index]; 
        //info!("{:?} Collapse: {:?}", node_index, node.identifier);

        match &mut node.data {
            NodeDataType::NotValid => {
                self.pending_collapses.push(template_node.level, node_index);

                match &template_node.value {
                    NodeTemplateValue::Groupe
                    | NodeTemplateValue::BuildHook => unreachable!(),
                    NodeTemplateValue::NumberRangeHook => {
                        return Ok(CollapseOperation::NumberRangeHook { 
                            index: node_index,
                            identifier: node.identifier,
                        });
                    },
                    NodeTemplateValue::NumberRange(number_range) => {
                        node.data = NodeDataType::NumberRange(number_range.to_owned());
                    },
                    NodeTemplateValue::PosSetHook => {
                        return Ok(CollapseOperation::PosSetHook { 
                            index: node_index,
                            identifier: node.identifier,
                        });
                    },
                    NodeTemplateValue::PosSet(position_set) => {
                        node.data = NodeDataType::PosSet(position_set.to_owned());
                    },
                }
                return Ok(CollapseOperation::None);
            }

            NodeDataType::NumberRange(number_data) => {
                if number_data.perm_counter >= number_data.permutation.max() as usize {
                    info!("{:?} Resetting Number faild", node_index);

                    number_data.perm_counter = 0;

                    let next_reset = node.next_reset;
                    self.reset_node(next_reset, template)?;

                    self.pending_collapses.push(template_node.level, node_index);

                    return Ok(CollapseOperation::None);
                }

                let value = number_data.permutation.get(number_data.perm_counter as _) as i32 + number_data.min;
                number_data.perm_counter += 1;

                number_data.value = value;
                info!("{:?} {:?}: {}", node_index, node.identifier, number_data.value);

                self.update_defined_by_number_range(node_index, template, value as usize)?;
            },
            NodeDataType::PosSet(pos_set) => {
                match &pos_set.rule {
                    PositionSetRule::GridInVolume(grid_data) => {

                        let mut new_positions = grid_data.volume.get_grid_positions(grid_data.spacing).collect::<Vec<_>>();
                        pos_set.positions.retain(|key, p| {
                            if let Some(i) = new_positions.iter().position(|t| *t == *p) {
                                new_positions.swap_remove(i);
                                true
                            } else {
                                false
                            }
                        });
                        let to_create_children = new_positions.iter()
                            .map(|p| pos_set.positions.insert(*p))
                            .collect::<Vec<_>>();
                        
                        self.upadte_defined_by_pos_set(node_index, &to_create_children, template, template_node)?;
                    },
                    PositionSetRule::GridOnPlane(grid_data) => {
                        let mut new_positions = grid_data.volume.get_grid_positions(grid_data.spacing)
                            .map(|p| vec3(p.x, p.y, grid_data.height))
                            .collect::<Vec<_>>();

                        pos_set.positions.retain(|key, p| {
                            if let Some(i) = new_positions.iter().position(|t| *t == *p) {
                                new_positions.swap_remove(i);
                                true
                            } else {
                                false
                            }
                        });
                        let to_create_children = new_positions.iter()
                            .map(|p| pos_set.positions.insert(*p))
                            .collect::<Vec<_>>();
                        
                        self.upadte_defined_by_pos_set(node_index, &to_create_children, template, template_node)?;
                    },
                }
            },
            NodeDataType::Build => {
                let identifier = node.identifier;
                self.create_defines_n(node_index, template)?;
                return Ok(CollapseOperation::BuildHook { 
                    index: node_index, 
                    identifier,
                });
            },
            NodeDataType::None => {},
        }

        self.create_defines_n(node_index, template)?;
        Ok(CollapseOperation::None)
    }

    pub fn pos_collapse_failed(&mut self, index: CollapseNodeKey) {
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        info!("{:?} Pos Collapse Faild {:?}", index, node.identifier);
        todo!();
    }

    pub fn collapse_failed(&mut self, index: CollapseNodeKey, template: &TemplateTree<T>) -> OctaResult<()> {        
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        info!("{:?} Collapse Faild {:?}", index, node.identifier);

        let level = template.nodes[node.template_index].level;

        self.pending_collapses.push(level, index);

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

    pub fn set_undo_data(&mut self, index: CollapseNodeKey, data: T::UndoData) -> OctaResult<()> {
        let node = self.nodes.get_mut(index)
            .ok_or(anyhow!("Index of build node to set data is not valid!"))?;

        node.undo_data = data;

        Ok(())
    }
}

impl<T: ModelGenerationTypes> TemplateTree<T> {
    pub fn get_collaper(&self) -> Collapser<T> {
        let inital_capacity = 1000;

        let mut collapser = Collapser{
            nodes: SlotMap::with_capacity_and_key(inital_capacity),
            pending_collapses: PendingOperations::new(self.max_level),
            pending_collapse_opperations: Vec::with_capacity(inital_capacity),
        };

        collapser.add_node(0, vec![], vec![], CollapseNodeKey::null(), CollapseChildKey::null(), self);
        collapser
    }
}
