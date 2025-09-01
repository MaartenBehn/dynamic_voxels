
use std::{collections::{HashMap, VecDeque}, fmt::{Debug, Octal}, iter, marker::PhantomData, mem, task::ready, usize};

use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, vec3a, IVec3, Vec3, Vec3Swizzles}, log::{debug, error, info}, vulkan::ash::vk::OpaqueCaptureDescriptorDataCreateInfoEXT, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};
use crate::{model::{composer::{build::{OnCollapseArgs, BS}, number_space::NumberSpaceTemplate, template::{ComposeTemplate, TemplateIndex}}, generation::pos_set::PositionSetRule}, util::{number::Nu, state_saver, vector::Ve}, volume::VolumeQureyPosValid};

use super::{number_space::NumberSpace, pending_operations::{PendingOperations, PendingOperationsRes}, position_space::PositionSpace};

new_key_type! { pub struct CollapseNodeKey; }
new_key_type! { pub struct CollapseChildKey; }

#[derive(Debug, Clone, Default)]
pub struct Collapser<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<V2, V3, T, B>>,
    pub pending: PendingOperations,
}

#[derive(Debug, Clone)]
pub struct CollapseNode<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub template_index: usize,
    pub level: usize,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>,
    pub defined_by: CollapseNodeKey,
    pub child_key: CollapseChildKey,
    pub data: NodeDataType<V2, V3, T, B>,
    pub next_reset: CollapseNodeKey,
}

#[derive(Debug, Clone)]
pub enum NodeDataType<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    NumberSet(NumberSpace<T>),
    PositionSpace(PositionSpace<V2, V3, T>),
    None,
    Build(B::CollapseValue)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateDefinesOperation {
    N{
        parent_index: CollapseNodeKey,
        defines_index: usize,
    },
    ByNode{
        parent_index: CollapseNodeKey,
        defines_index: usize,
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    pub fn new(template: &ComposeTemplate<V2, V3, T, B>, state: &mut B) -> Self {
        let inital_capacity = 1000;

        let mut collapser = Self {
            nodes: SlotMap::with_capacity_and_key(inital_capacity),
            pending: PendingOperations::new(template.max_level),
        };

        collapser.add_node(
            0, 
            vec![], 
            CollapseNodeKey::null(), 
            CollapseChildKey::null(), 
            template,
            state);

        collapser.template_changed(template, state); 
        collapser
    }
 
    pub fn run(&mut self, template: &ComposeTemplate<V2, V3, T, B>, state: &mut B) {

        loop {
            match self.pending.pop() {
                PendingOperationsRes::Collapse(key) => self.collapse_node(key, template, state),
                PendingOperationsRes::CreateDefined(operation) => self.upadte_defined(
                    operation, template, state),
                PendingOperationsRes::Empty => break,
            }
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate<V2, V3, T, B>, state: &mut B) {
        let node = &mut self.nodes[node_index];
        let template_node = &template.nodes[node.template_index]; 
        //info!("{:?} Collapse: {:?}", node_index, node.identifier);

        match &mut node.data {
            NodeDataType::NumberSet(space) => {
                if space.update().is_err() {
                    panic!("{:?} Collapse Number faild", node_index);
                }
            },
            NodeDataType::PositionSpace(space) => space.update(),
            NodeDataType::None => {},
            NodeDataType::Build(t) => B::on_collapse(OnCollapseArgs { 
                collapse_index: node_index,
                collapser: &self, 
                template: template, 
                state
            }),
        }

        self.push_defined(node_index, template);
    }

    pub fn get_number(&self, index: CollapseNodeKey) -> T {
        let node = self.nodes.get(index).expect("Number Set by index not found");
        
        match &node.data {
            NodeDataType::NumberSet(d) => d.value,
            _ => panic!("Template Node {:?} is not of Type Number Space", node.template_index)
        }
    }

    pub fn get_position<V: Ve<T, D>, const D: usize>(&self, index: CollapseNodeKey) -> V {
        let node = &self.nodes[index];
        let parent = &self.nodes[node.defined_by];
        
        match &parent.data {
            NodeDataType::PositionSpace(d) => d.get_position(node.child_key),
            _ => panic!("Template Node {:?} is not of Type Position Space Set", node.template_index)
        }
    }
 
    fn get_dependend_index(
        &self, 
        template_index: TemplateIndex,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)],
        collapser: &Collapser<V2, V3, T, B>
    ) -> CollapseNodeKey {
        depends.iter().find(|(i, _)| *i == template_index)
            .expect(&format!("Node does not depend on {:?}", template_index)).1[0]
    }

    pub fn get_dependend_number(
        &self, 
        template_index: TemplateIndex,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)],
        collapser: &Collapser<V2, V3, T, B>
    ) -> T { 
        collapser.get_number(self.get_dependend_index(template_index, depends, collapser))
    }

    pub fn get_dependend_position<V: Ve<T, D>, const D: usize>(
        &self, 
        template_index: TemplateIndex,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)],
        collapser: &Collapser<V2, V3, T, B>
    ) -> V { 
        collapser.get_position(self.get_dependend_index(template_index, depends, collapser))
    }

    pub fn get_root_key(&self) -> CollapseNodeKey {
        self.nodes.keys().next().unwrap()
    }
}

impl UpdateDefinesOperation {
    pub fn get_parent_index(&self) -> CollapseNodeKey {
        match self {
            UpdateDefinesOperation::N { parent_index, .. }
            | UpdateDefinesOperation::ByNode { parent_index, .. } => *parent_index,
        }
    }
}
