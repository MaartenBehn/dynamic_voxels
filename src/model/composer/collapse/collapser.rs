
use std::{collections::{HashMap, VecDeque}, fmt::{Debug, Octal}, iter, marker::PhantomData, mem, task::ready, usize};

use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, vec3a, IVec3, Vec3}, log::{debug, error, info}, vulkan::ash::vk::OpaqueCaptureDescriptorDataCreateInfoEXT, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};
use crate::{model::{composer::{number_space::NumberSpace, template::{ComposeTemplate, TemplateIndex}}, generation::pos_set::PositionSetRule}, volume::VolumeQureyPosValid};

use super::{number_space::NumberSet, pending_operations::{PendingOperations, PendingOperationsRes}, position_space::PositionSet};

new_key_type! { pub struct CollapseNodeKey; }
new_key_type! { pub struct CollapseChildKey; }

#[derive(Debug, Clone, Default)]
pub struct Collapser {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode>,
    pub pending: PendingOperations,
}

#[derive(Debug, Clone)]
pub struct CollapseNode {
    pub template_index: usize,
    pub level: usize,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>,
    pub defined_by: CollapseNodeKey,
    pub child_key: CollapseChildKey,
    pub data: NodeDataType,
    pub next_reset: CollapseNodeKey,
}

#[derive(Debug, Clone)]
pub enum NodeDataType {
    NumberSet(NumberSet),
    PositionSpace(PositionSet),
    None,
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

impl Collapser {
    pub fn run(&mut self, template: &ComposeTemplate) {

        loop {
            match self.pending.pop() {
                PendingOperationsRes::Collapse(key) => self.collapse_node(key, template),
                PendingOperationsRes::CreateDefined(operation) => self.create_defined(operation, template),
                PendingOperationsRes::Empty => break,
            }
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate) {
        let node = &mut self.nodes[node_index];
        let template_node = &template.nodes[node.template_index]; 
        //info!("{:?} Collapse: {:?}", node_index, node.identifier);

        match &mut node.data { 
            NodeDataType::NumberSet(space) => {
                if space.next_value().is_err() {
                    info!("{:?} Resetting Number faild", node_index);

                    let next_reset = node.next_reset;
                    self.reset_node(next_reset, template);

                    self.pending.push_collpase(template_node.level, node_index);

                    return;
                }

                let value = space.value;
                info!("{:?} {:?}: {}", node_index, node.identifier, value);
                self.update_defined_by_number_range(node_index, template, value as usize);
            },
            NodeDataType::PositionSpace(space) => {

                let mut new_positions = match &space.rule {
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

                space.positions.retain(|key, p| {
                    if let Some(i) = new_positions.iter().position(|t| *t == *p) {
                        new_positions.swap_remove(i);
                        true
                    } else {
                        false
                    }
                });
                let to_create_children = new_positions.iter()
                    .map(|p| space.positions.insert(*p))
                    .collect::<Vec<_>>();

                self.update_defined_by_pos_set(node_index, to_create_children, template, template_node);
            },
            NodeDataType::None => {},
        }

        self.update_defines_n(node_index, template);
    }

    pub fn get_number(&self, index: CollapseNodeKey) -> i32 {
        let node = self.nodes.get(index).expect("Number Set by index not found");
        
        match &node.data {
            NodeDataType::NumberSet(d) => d.value,
            _ => panic!("Template Node {:?} is not of Type Number Set", node.template_index)
        }
    }

    pub fn get_dependend_number(
        &self, 
        template_index: TemplateIndex,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)],
        collapser: &Collapser
    ) -> i32 {
        let index = depends.iter().find(|(i, _)| *i == template_index)
            .expect(&format!("Node does not depend on {:?}", template_index)).1[0];

        collapser.get_number(index)
    }
}



impl ComposeTemplate {
    pub fn get_collaper(&self) -> Collapser {
        let inital_capacity = 1000;

        let mut collapser = Collapser{
            nodes: SlotMap::with_capacity_and_key(inital_capacity),
            pending: PendingOperations::new(self.max_level),
        };

        collapser.add_node(0, vec![], vec![], vec![], CollapseNodeKey::null(), CollapseChildKey::null(), self);
        collapser
    }
}

impl CreateDefinesOperation {
    pub fn get_parent_index(&self) -> CollapseNodeKey {
        match self {
            CreateDefinesOperation::CreateN { parent_index, .. }
            | CreateDefinesOperation::CreateByNumberRange { parent_index, .. }
            | CreateDefinesOperation::CreateByPosSet { parent_index, .. } => *parent_index,
        }
    }
}
