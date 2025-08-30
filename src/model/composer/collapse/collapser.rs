
use std::{collections::{HashMap, VecDeque}, fmt::{Debug, Octal}, iter, marker::PhantomData, mem, task::ready, usize};

use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, vec3a, IVec3, Vec3, Vec3Swizzles}, log::{debug, error, info}, vulkan::ash::vk::OpaqueCaptureDescriptorDataCreateInfoEXT, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};
use crate::{model::{composer::{number_space::NumberSpaceTemplate, template::{ComposeTemplate, TemplateIndex}}, generation::pos_set::PositionSetRule}, util::{number::Nu, vector::Ve}, volume::VolumeQureyPosValid};

use super::{number_space::NumberSpace, pending_operations::{PendingOperations, PendingOperationsRes}, position_space::PositionSpace};

new_key_type! { pub struct CollapseNodeKey; }
new_key_type! { pub struct CollapseChildKey; }

#[derive(Debug, Clone, Default)]
pub struct Collapser<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<V2, V3, T>>,
    pub pending: PendingOperations,
}

#[derive(Debug, Clone)]
pub struct CollapseNode<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    pub template_index: usize,
    pub level: usize,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>,
    pub defined_by: CollapseNodeKey,
    pub child_key: CollapseChildKey,
    pub data: NodeDataType<V2, V3, T>,
    pub next_reset: CollapseNodeKey,
}

#[derive(Debug, Clone)]
pub enum NodeDataType<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    NumberSet(NumberSpace<T>),
    PositionSpace(PositionSpace<V2, V3, T>),
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

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> Collapser<V2, V3, T> {
    pub fn run(&mut self, template: &ComposeTemplate<V2, V3, T>) {

        loop {
            match self.pending.pop() {
                PendingOperationsRes::Collapse(key) => self.collapse_node(key, template),
                PendingOperationsRes::CreateDefined(operation) => self.create_defined(operation, template),
                PendingOperationsRes::Empty => break,
            }
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate<V2, V3, T>) {
        let node = &mut self.nodes[node_index];
        let template_node = &template.nodes[node.template_index]; 
        //info!("{:?} Collapse: {:?}", node_index, node.identifier);

        match &mut node.data { 
            NodeDataType::NumberSet(space) => {
                if space.next_value().is_err() {
                    panic!("{:?} Collapse Number faild", node_index);
                }

                let value = space.value;
                info!("{:?} NumberSpace: {:?}: {:?}", node_index, node.template_index, value);
                //self.update_defined_by_number_range(node_index, template, value as usize);
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

                //self.update_defined_by_pos_set(node_index, to_create_children, template, template_node);
            },
            NodeDataType::None => {},
        }

        //self.update_defines_n(node_index, template);
    }

    pub fn get_number(&self, index: CollapseNodeKey) -> T {
        let node = self.nodes.get(index).expect("Number Set by index not found");
        
        match &node.data {
            NodeDataType::NumberSet(d) => d.value,
            _ => panic!("Template Node {:?} is not of Type Number Set", node.template_index)
        }
    }

    pub fn get_position<V: Ve<T, D>, const D: usize>(&self, index: CollapseNodeKey) -> V {
        let node = &self.nodes[index];
        let parent = &self.nodes[node.defined_by];
        
        match &parent.data {
            NodeDataType::PositionSpace(d) => d.get_position(node.child_key),
            _ => panic!("Template Node {:?} is not of Type PositionSpace Set", node.template_index)
        }
    }
 
    fn get_dependend_index(
        &self, 
        template_index: TemplateIndex,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)],
        collapser: &Collapser<V2, V3, T>
    ) -> CollapseNodeKey {
        depends.iter().find(|(i, _)| *i == template_index)
            .expect(&format!("Node does not depend on {:?}", template_index)).1[0]
    }

    pub fn get_dependend_number(
        &self, 
        template_index: TemplateIndex,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)],
        collapser: &Collapser<V2, V3, T>
    ) -> T { 
        collapser.get_number(self.get_dependend_index(template_index, depends, collapser))
    }

    pub fn get_dependend_position<V: Ve<T, D>, const D: usize>(
        &self, 
        template_index: TemplateIndex,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)],
        collapser: &Collapser<V2, V3, T>
    ) -> V { 
        collapser.get_position(self.get_dependend_index(template_index, depends, collapser))
    }
}


impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> ComposeTemplate<V2, V3, T> {
    pub fn get_collaper(&self) -> Collapser<V2, V3, T> {
        let inital_capacity = 1000;

        let mut collapser = Collapser{
            nodes: SlotMap::with_capacity_and_key(inital_capacity),
            pending: PendingOperations::new(self.max_level),
        };

        collapser.add_node(0, vec![], CollapseNodeKey::null(), CollapseChildKey::null(), self);
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
