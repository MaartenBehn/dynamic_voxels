
use std::{collections::{HashMap, VecDeque}, fmt::{Debug, Octal}, iter, marker::PhantomData, mem, task::ready, usize};

use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, vec3a, IVec3, Vec3}, log::{debug, error, info}, vulkan::ash::vk::OpaqueCaptureDescriptorDataCreateInfoEXT, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};


use crate::{model::generation::pos_set::PositionSetRule, volume::{VolumeQureyPosValid, VolumeQureyPosValid2D}};

use super::{builder::{BuilderNode, ModelSynthesisBuilder}, number_range::NumberRange, pending_operations::PendingOperations, pos_set::PositionSet, relative_path::{LeafType, RelativePathTree}, template::{NodeTemplateValue, TemplateIndex, TemplateNode, TemplateTree}, traits::ModelGenerationTypes};

new_key_type! { pub struct CollapseNodeKey; }
new_key_type! { pub struct CollapseChildKey; }

#[derive(Debug, Clone)]
pub struct Collapser<T: ModelGenerationTypes> {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<T>>,
    pub pending_collapses: PendingOperations<CollapseNodeKey>,
    pub pending_create_defines: PendingOperations<CreateDefinesOperation>,
    pub pending_user_opperations: VecDeque<CollapseOperation<T>>,
}

#[derive(Debug, Clone)]
pub struct CollapseNode<T: ModelGenerationTypes> {
    pub template_index: usize,
    pub identifier: T::Identifier,
    pub level: usize,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub restricts: Vec<(T::Identifier, Vec<CollapseNodeKey>)>,
    pub depends: Vec<(T::Identifier, Vec<CollapseNodeKey>)>,
    pub knows: Vec<(T::Identifier, Vec<CollapseNodeKey>)>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateDefinesOperation {
    CreateN{
        parent_index: CollapseNodeKey,
        ammount_index: usize,
    },
    CreateByNumberRange{
        parent_index: CollapseNodeKey,
        by_value_index: usize,
        ammount: usize,
    },
    CreateByPosSet{
        parent_index: CollapseNodeKey,
        by_value_index: usize,
        to_create_children: Vec<CollapseChildKey>,
    }
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
    RestrictHook {
        index: CollapseNodeKey,
        identifier: T::Identifier,
        restricts_index: CollapseNodeKey,
        restricts_identifier: T::Identifier,
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
    -> Option<(CollapseOperation<T>, &mut Collapser<T>)> {

        if let Some(opperation) = self.pending_user_opperations.pop_front() {
            return Some((opperation, self));
        }

        if let Some(opperation) = self.pending_create_defines.pop() {
            self.create_defined(opperation, template);
        }

        if let Some(key) = self.pending_collapses.pop() {
            self.collapse_node(key, template);
        }
 
        if let Some(opperation) = self.pending_user_opperations.pop_front() {
            return Some((opperation, self));
        } else {
            None
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>) {
        let node = &mut self.nodes[node_index];
        let template_node = &template.nodes[node.template_index]; 
        //info!("{:?} Collapse: {:?}", node_index, node.identifier);

        match &mut node.data {
            NodeDataType::NotValid => {
                match &template_node.value {
                    NodeTemplateValue::Groupe
                    | NodeTemplateValue::BuildHook => unreachable!(),
                    NodeTemplateValue::NumberRangeHook => {
                        self.pending_user_opperations.push_back(CollapseOperation::NumberRangeHook { 
                            index: node_index,
                            identifier: node.identifier,
                        });
                    },
                    NodeTemplateValue::NumberRange(number_range) => {
                        node.data = NodeDataType::NumberRange(number_range.to_owned());
                    },
                    NodeTemplateValue::PosSetHook => {
                        self.pending_user_opperations.push_back(CollapseOperation::PosSetHook { 
                            index: node_index,
                            identifier: node.identifier,
                        });
                    },
                    NodeTemplateValue::PosSet(position_set) => {
                        node.data = NodeDataType::PosSet(position_set.to_owned());
                    },
                }

                self.pending_collapses.push(template_node.level, node_index);
                return;
            }

            NodeDataType::NumberRange(number_data) => {
                if number_data.next_value().is_err() {
                    info!("{:?} Resetting Number faild", node_index);

                    let next_reset = node.next_reset;
                    self.reset_node(next_reset, template);

                    self.pending_collapses.push(template_node.level, node_index);

                    return;
                }

                let value = number_data.value;
                info!("{:?} {:?}: {}", node_index, node.identifier, value);
                self.update_defined_by_number_range(node_index, template, value as usize);
                self.push_restricts_collapse_opperations(node_index, template);
            },
            NodeDataType::PosSet(pos_set) => {

                let mut new_positions = match &pos_set.rule {
                    PositionSetRule::GridInVolume(grid_data) => {
                        grid_data.volume.get_grid_positions(grid_data.spacing).collect::<Vec<_>>()
                    },
                    PositionSetRule::GridOnPlane(grid_data) => {
                        grid_data.volume.get_grid_positions(grid_data.spacing)
                            .map(|p| vec3a(p.x, p.y, grid_data.height))
                            .collect::<Vec<_>>()
                    },
                    PositionSetRule::Path(path) => path.get_positions(),
                };

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

                self.update_defined_by_pos_set(node_index, to_create_children, template, template_node);

                self.push_restricts_collapse_opperations(node_index, template);
            },
            NodeDataType::Build => {
                let identifier = node.identifier;
                self.push_restricts_collapse_opperations(node_index, template);

                self.pending_user_opperations.push_back(CollapseOperation::BuildHook { 
                    index: node_index, 
                    identifier,
                });
            },
            NodeDataType::None => {},
        }

        self.update_defines_n(node_index, template);
    }
 
    pub fn pos_collapse_failed(&mut self, index: CollapseNodeKey) {
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        info!("{:?} Pos Collapse Faild {:?}", index, node.identifier);
        todo!();
    }

    pub fn collapse_failed(&mut self, index: CollapseNodeKey, template: &TemplateTree<T>) {        
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        info!("{:?} Collapse Faild {:?}", index, node.identifier);

        let level = template.nodes[node.template_index].level;

        self.pending_collapses.push(level, index);

        let mut last = CollapseNodeKey::null();
        for (_, is) in node.depends.to_owned().into_iter().rev() {
            for i in is {
                if last.is_null() {
                    self.reset_node(i, template);
                } else {
                    self.set_next_reset(last, i); 
                }

                last = i;
            }
        }
    }
}

impl<T: ModelGenerationTypes> TemplateTree<T> {
    pub fn get_collaper(&self) -> Collapser<T> {
        let inital_capacity = 1000;

        let mut collapser = Collapser{
            nodes: SlotMap::with_capacity_and_key(inital_capacity),
            pending_collapses: PendingOperations::new(self.max_level),
            pending_create_defines: PendingOperations::new(self.max_level),
            pending_user_opperations: VecDeque::with_capacity(inital_capacity),
        };

        collapser.add_node(0, vec![], vec![], vec![], CollapseNodeKey::null(), CollapseChildKey::null(), self);
        collapser
    }
}
