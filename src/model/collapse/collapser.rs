use std::{collections::{HashMap, VecDeque}, fmt::{Debug, Octal}, iter, marker::PhantomData, mem::{self, ManuallyDrop}, task::ready, usize};

use itertools::{Either, Itertools};
use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, vec3a, IVec3, Vec3, Vec3Swizzles}, log::{debug, error, info}, vulkan::ash::vk::OpaqueCaptureDescriptorDataCreateInfoEXT, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};
use crate::{model::{composer::{build::{OnCollapseArgs, BS}, template::{ComposeTemplate, ComposeTemplateValue, TemplateIndex}}}, util::{number::Nu, state_saver, vector::Ve}, volume::VolumeQureyPosValid};

use super::{add_nodes::{GetNewChildrenData, GetValueData}, number_space::NumberSpace, pending_operations::{PendingOperations, PendingOperationsRes}, position_space::PositionSpace};

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
    pub child_keys: Vec<(CollapseNodeKey, CollapseChildKey)>,
    pub data: NodeDataType<V2, V3, T, B>,
    pub next_reset: CollapseNodeKey,
}

#[derive(Debug, Clone)]
pub enum NodeDataType<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    NumberSet(NumberSpace<T>),
    PositionSpace2D(PositionSpace<V2, T, 2>),
    PositionSpace3D(PositionSpace<V3, T, 3>),
    None,
    Build(B::CollapseValue)
}

union VUnion<VA: Ve<T, DA>, VB: Ve<T, DB>, T: Nu, const DA: usize, const DB: usize> {
    a: VA,
    b: VB,
    p: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateDefinesOperation {
    One {
        template_index: TemplateIndex,
        parent_index: CollapseNodeKey,
    },
    Creates {
        template_index: TemplateIndex,
        parent_index: CollapseNodeKey,
        creates_index: usize,
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    pub async fn new(template: &ComposeTemplate<V2, V3, T, B>, state: &mut B) -> Self {
        let inital_capacity = 1000;

        let mut collapser = Self {
            nodes: SlotMap::with_capacity_and_key(inital_capacity),
            pending: PendingOperations::new(template.max_level),
        };

        collapser.add_node(
            0, 
            vec![], 
            CollapseNodeKey::null(), 
            vec![], 
            template,
            state).await;

        collapser.template_changed(template, state); 
        collapser
    }
 
    pub async fn run(&mut self, template: &ComposeTemplate<V2, V3, T, B>, state: &mut B) {

        loop {
            match self.pending.pop() {
                PendingOperationsRes::Collapse(key) => self.collapse_node(key, template, state).await,
                PendingOperationsRes::CreateDefined(operation) => self.upadte_defined(
                    operation, template, state).await,
                PendingOperationsRes::Retry => {
                    info!("Swaped Next Collapse");
                },
                PendingOperationsRes::Empty => break,
            }
        }
    }

    async fn collapse_node(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate<V2, V3, T, B>, state: &mut B) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index]; 
        info!("{:?} Collapse", node_index);

        let get_value_data = GetValueData { 
            defined_by: node.defined_by, 
            child_indexs: &node.child_keys, 
            depends: &node.depends, 
            depends_loop: &template_node.depends_loop,
            index: node_index,
        }; 

        let needs_recompute = match &template_node.value {
            ComposeTemplateValue::None => false,
            ComposeTemplateValue::NumberSpace(space) => {
                let (new_val, r) = space.get_value(get_value_data, &self);
 
                let data = match &mut self.nodes[node_index].data {
                    NodeDataType::NumberSet(space) => space,
                    _ => unreachable!()
                };
                data.update(new_val); 

                r
            },

            ComposeTemplateValue::PositionSpace2D(space) => {
                let (new_positions, r) = space.get_value(get_value_data, &self);
                let new_positions = new_positions.collect_vec();

                let data = match &mut self.nodes[node_index].data {
                    NodeDataType::PositionSpace2D(space) => space,
                    _ => unreachable!()
                };
                data.update(new_positions); 

                r
            },

            ComposeTemplateValue::PositionSpace3D(space) => {
                let (new_positions, r) = space.get_value(get_value_data, &self);
                let new_positions = new_positions.collect_vec();

                let data = match &mut self.nodes[node_index].data {
                    NodeDataType::PositionSpace3D(space) => space,
                    _ => unreachable!()
                };
                data.update(new_positions); 

                r
            },

            ComposeTemplateValue::Build(t) => {
                let data = match &node.data {
                    NodeDataType::Build(build) => build,
                    _ => unreachable!()
                };

                let build = B::on_collapse(OnCollapseArgs { 
                    template_value: t,
                    collapse_node: node,
                    collapse_value: data,
                    get_value_data,
                    collapser: &self, 
                    template, 
                    state,
                }).await;

                self.nodes[node_index].data = NodeDataType::Build(build);

                false
            },
        };   

        if needs_recompute {
            self.pending.push_later_collpase(template_node.level, node_index);
        }

        self.push_defined(node_index, template);
        self.collapse_all_childeren(node_index, template);
    }

    fn collapse_all_childeren(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate<V2, V3, T, B>) {
        let node = &self.nodes[node_index];

        for (_, list) in node.children.iter() {
            for child_index in list.iter() {

                let child_node = &self.nodes[*child_index];
                let template_node = &template.nodes[child_node.template_index]; 
                self.pending.push_collpase(template_node.level, *child_index);
            }
        }
    }

    pub fn get_new_children(&self, index: CollapseNodeKey) -> impl Iterator<Item = (CollapseNodeKey, CollapseChildKey)> {
        let node = self.nodes.get(index)
            .expect("New Children Node not found");

        let keys = match &node.data {
            NodeDataType::NumberSet(number_space) => {
                todo!()
            },
            NodeDataType::PositionSpace2D(position_space) => position_space.get_new_children(),
            NodeDataType::PositionSpace3D(position_space) => position_space.get_new_children(),
            NodeDataType::Build(_) 
            | NodeDataType::None 
                => panic!("Called get new children on node that has no children"),
        };

        keys.iter()
            .map(move |k| (index, *k))
    }

    pub fn is_child_valid(
        &self, 
        index: CollapseNodeKey,
        child_index: CollapseChildKey,
    ) -> bool {
        let node = self.nodes.get(index)
            .expect("Is Child Valid Node not found");

        match &node.data {
            NodeDataType::PositionSpace2D(position_space) => position_space.is_child_valid(child_index),
            NodeDataType::PositionSpace3D(position_space) => position_space.is_child_valid(child_index),
            NodeDataType::NumberSet(_)
            | NodeDataType::Build(_) 
            | NodeDataType::None 
                => panic!("Called is child valid on node that has no children"),
        }
    }

    pub fn get_number(&self, index: Option<CollapseNodeKey>) -> (T, bool) {
        if index.is_none() {
            return (T::ZERO, true);
        }

        let node = self.nodes.get(index.unwrap())
            .expect("Number Set by index not found");
        
        let v = match &node.data {
            NodeDataType::NumberSet(d) => d.value,
            _ => panic!("Template Node {:?} is not of Type Number Space", node.template_index)
        };

        (v, false)
    }

    pub fn get_position<V: Ve<T, D>, const D: usize>(&self, parent_index: CollapseNodeKey, child_index: CollapseChildKey) -> (V, bool) {
        let parent = &self.nodes[parent_index];
       
        let v = match D {
            2 => {
                let v = match &parent.data {
                    NodeDataType::PositionSpace2D(d) => d.get_position(child_index),
                    _ => panic!("Template Node {:?} is not of Type Position Space 2D Set", parent.template_index)
                };

                // Safety V2 and V are the same Type bause D == 2
                unsafe { VUnion{ a: v }.b }
            }
            3 => {
                let v = match &parent.data {
                    NodeDataType::PositionSpace3D(d) => d.get_position(child_index),
                    _ => panic!("Template Node {:?} is not of Type Position Space 3D Set", parent.template_index)
                };

                // Safety V3 and V are the same Type bause D == 3
                unsafe { VUnion{ a: v }.b }
            }
            _ => unreachable!()
        };

        (v, false)
    }

    pub fn get_position_set<V: Ve<T, D>, const D: usize>(&self, index: Option<CollapseNodeKey>) -> (impl Iterator<Item = V>, bool) {
        if index.is_none() {
            return (Either::Left(iter::empty()), true);
        }

        let node = &self.nodes[index.unwrap()];
       
        let set = match D {
            2 => {
                let i = match &node.data {
                    NodeDataType::PositionSpace2D(d) => d.get_positions(),
                    _ => panic!("Template Node {:?} is not of Type Position Space Set 2D", node.template_index)
                };
                
                Either::Left(i.map(|v|  {
                    // Safety V2 and V are the same Type bause D == 2
                    unsafe { VUnion{ a: v }.b }
                }))
            }
            3 => {
                let i = match &node.data {
                    NodeDataType::PositionSpace3D(d) => d.get_positions(),
                    _ => panic!("Template Node {:?} is not of Type Position Space Set 3D", node.template_index)
                };

                Either::Right(i.map(|v| {
                    // Safety V3 and V are the same Type bause D == 3
                    unsafe { VUnion{ a: v }.b }
                }))
            }
            _ => unreachable!()
        };

        (Either::Right(set), false)
    }

    pub fn get_dependend_new_children(
        &self, 
        template_index: TemplateIndex,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)],
    ) -> impl Iterator<Item = (CollapseNodeKey, CollapseChildKey)> {

        let (_, keys) = depends.iter()
            .find(|(i, _)| *i == template_index)
            .expect("New Child node not in depends");

        keys.iter()
            .map(|key| self.get_new_children(*key))
            .flatten()
    }
 
    fn get_dependend_index_value_data(
        &self, 
        template_index: TemplateIndex,
        get_value_data: GetValueData,
    ) -> Option<CollapseNodeKey> {
        if let Some((_, keys)) = get_value_data.depends.iter().find(|(i, _)| *i == template_index) {
            return Some(keys[0]);
        }

        let (_, loop_path) = get_value_data.depends_loop.iter().find(|(i, _)| *i == template_index)
            .expect(&format!("{template_index} must be in depends or depends_loop"));

        let keys = self.get_depends_from_loop_path(get_value_data, loop_path);
        if keys.is_empty() {
            None
        } else {
            Some(keys[0])
        }
    }

    fn get_child_index(
        &self, 
        index: CollapseNodeKey,
        get_value_data: GetValueData,
    ) -> CollapseChildKey {
        let (_, ci) = get_value_data.child_indexs.iter()
            .find(|(i, _)| *i == index) 
            .expect("Could not find child index");

        *ci
    }

      
    pub fn get_dependend_number(
        &self, 
        template_index: TemplateIndex,
        get_value_data: GetValueData,
    ) -> (T, bool) { 
        self.get_number(self.get_dependend_index_value_data(template_index, get_value_data))
    }

    pub fn get_dependend_position_set<V: Ve<T, D>, const D: usize>(
        &self, 
        template_index: TemplateIndex,
        get_value_data: GetValueData,
    ) -> (impl Iterator<Item = V>, bool) { 
        self.get_position_set(self.get_dependend_index_value_data(template_index, get_value_data))
    }
 
    pub fn get_dependend_position<V: Ve<T, D>, const D: usize>(
        &self, 
        template_index: TemplateIndex,
        get_value_data: GetValueData,
    ) -> (V, bool) { 
        let index = self.get_dependend_index_value_data(template_index, get_value_data);
        
        if index.is_none() {
            return (V::ZERO, true);
        }
        let index = index.unwrap();
        let child_index = self.get_child_index(index, get_value_data);

        self.get_position(index, child_index)
    }

    pub fn get_root_key(&self) -> CollapseNodeKey {
        self.nodes.keys().next().unwrap()
    }
}

impl UpdateDefinesOperation {
    pub fn get_parent_index(&self) -> CollapseNodeKey {
        match self {
            UpdateDefinesOperation::One { parent_index, .. }
            | UpdateDefinesOperation::Creates { parent_index, .. } => *parent_index,
        }
    }
}
