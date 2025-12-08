use std::{collections::{HashMap, VecDeque}, fmt::{Debug, Octal}, iter, marker::PhantomData, mem::{self, ManuallyDrop}, task::ready, usize};

use itertools::{Either, Itertools};
use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, vec3a, IVec3, Vec3, Vec3Swizzles}, log::{debug, error, info}, vulkan::ash::vk::OpaqueCaptureDescriptorDataCreateInfoEXT, OctaResult};
use rayon::iter::IntoParallelIterator;
use slotmap::{new_key_type, Key, SlotMap};
use smallvec::SmallVec;
use crate::{model::{composer::build::{OnCollapseArgs, BS}, template::{value::TemplateValue, Template, TemplateIndex}}, util::{iter_merger::{IM2, IM3}, number::Nu, state_saver, vector::Ve}, volume::VolumeQureyPosValid};

use super::{add_nodes::{GetNewChildrenData, GetValueData}, external_input::ExternalInput, number_set::NumberSet, pending_operations::{PendingOperations, PendingOperationsRes}, position_pair_set::PositionPairSet, position_set::PositionSet};

new_key_type! { pub struct CollapseNodeKey; }
new_key_type! { pub struct CollapseChildKey; }

#[derive(Debug, Clone, Default)]
pub struct Collapser<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<V2, V3, T, B>>,
    pub nodes_per_template_index: Vec<SmallVec<[CollapseNodeKey; 4]>>,
    pub pending: PendingOperations,
}

#[derive(Debug, Clone)]
pub struct CollapseNode<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub template_index: usize,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>,
    pub defined_by: CollapseNodeKey,
    pub child_keys: Vec<(CollapseNodeKey, CollapseChildKey)>,
    pub data: NodeDataType<V2, V3, T, B>,
    pub next_reset: CollapseNodeKey,
}

#[derive(Debug, Clone)]
pub enum NodeDataType<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    NumberSet(NumberSet<T>),
    PositionSet2D(PositionSet<V2, T, 2>),
    PositionSet3D(PositionSet<V3, T, 3>),
    PositionPairSet2D(PositionPairSet<V2, T, 2>),
    PositionPairSet3D(PositionPairSet<V3, T, 3>),
    None,
    Build(B::CollapseValue)
}

union VUnion<VA: Ve<T, DA>, VB: Ve<T, DB>, T: Nu, const DA: usize, const DB: usize> {
    a: VA,
    b: VB,
    p: PhantomData<T>,
}

union PairUnion<VA: Ve<T, DA>, VB: Ve<T, DB>, T: Nu, const DA: usize, const DB: usize> {
    a: (VA, VA),
    b: (VB, VB),
    p: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateDefinesOperation { 
    pub template_index: TemplateIndex,
    pub parent_index: CollapseNodeKey,
    pub creates_index: usize,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    pub async fn new(template: &Template<V2, V3, T, B>, state: &mut B) -> Self {
        let inital_capacity = 1000;

        let mut collapser = Self {
            nodes: SlotMap::with_capacity_and_key(inital_capacity),
            pending: PendingOperations::new(template.max_level),
            nodes_per_template_index: vec![SmallVec::new(); template.nodes.len()],
        };

        collapser.add_node(
            0, 
            vec![], 
            CollapseNodeKey::null(), 
            vec![], 
            template,
            state).await;

        collapser.pending.push_collpase(1, collapser.get_root_key());

        dbg!(&collapser);
        dbg!(&template);
        collapser
    }
 
    pub async fn run(
        &mut self, 
        template: &Template<V2, V3, T, B>, 
        state: &mut B,
        external_input: ExternalInput,

    ) {

        loop {
            match self.pending.pop() {
                PendingOperationsRes::Collapse(key) => self.collapse_node(
                    key, template, state, external_input).await,

                PendingOperationsRes::CreateDefined(operation) => self.update_defined(
                    operation, template, state).await,

                PendingOperationsRes::Retry => {
                    #[cfg(debug_assertions)]
                    info!("Swaped Next Collapse");
                },
                PendingOperationsRes::Empty => break,
            }
        }
    }

    async fn collapse_node(
        &mut self, 
        node_index: CollapseNodeKey, 
        template: &Template<V2, V3, T, B>, 
        state: &mut B, 
        external_input: ExternalInput,
    ) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index]; 
        let value = &template.values[template_node.value_index];

        #[cfg(debug_assertions)]
        info!("{:?} Collapse", node_index);

        let get_value_data = GetValueData { 
            defined_by: node.defined_by, 
            child_indexs: &node.child_keys, 
            depends: &node.depends, 
            depends_loop: &template_node.depends_loop,
            index: node_index,
            external_input,
        }; 

        let needs_recompute = match value {
            TemplateValue::None => false,
            TemplateValue::NumberSet(space) => {
                todo!();
                let (mut new_val, r) = space.get_value(get_value_data, &self, template);
                let new_val = new_val.next().unwrap(); 

                let data = match &mut self.nodes[node_index].data {
                    NodeDataType::NumberSet(space) => space,
                    _ => unreachable!()
                };

                // TODO random select
                data.update(new_val); 

                r
            },
            TemplateValue::PositionSet2D(position_set_template) => {
                let (new_positions, r) = position_set_template.get_value(get_value_data, &self, template);

                let data = match &mut self.nodes[node_index].data {
                    NodeDataType::PositionSet2D(space) => space,
                    _ => unreachable!()
                };
                data.update(new_positions); 

                r
            },
            TemplateValue::PositionSet3D(position_set_template) => {
                let (new_positions, r) = position_set_template.get_value(get_value_data, &self, template);

                let data = match &mut self.nodes[node_index].data {
                    NodeDataType::PositionSet3D(space) => space,
                    _ => unreachable!()
                };
                data.update(new_positions); 

                r
            },
            TemplateValue::PositionPairSet2D(position_set_template) => {
                let (new_positions, r) = position_set_template.get_value(get_value_data, &self, template);

                let data = match &mut self.nodes[node_index].data {
                    NodeDataType::PositionPairSet2D(space) => space,
                    _ => unreachable!()
                };
                data.update(new_positions); 

                r
            },
            TemplateValue::PositionPairSet3D(position_set_template) => {
                let (new_positions, r) = position_set_template.get_value(get_value_data, &self, template);

                let data = match &mut self.nodes[node_index].data {
                    NodeDataType::PositionPairSet3D(space) => space,
                    _ => unreachable!()
                };
                data.update(new_positions); 

                r
            },
            TemplateValue::Build(t) => {
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
            _ => unreachable!(),
        };   

        if needs_recompute {
            self.pending.push_later_collpase(template_node.level, node_index);
        }

        self.push_defined(node_index, template);
        self.collapse_all_childeren(node_index, template);
    }

    fn collapse_all_childeren(&mut self, node_index: CollapseNodeKey, template: &Template<V2, V3, T, B>) {
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
            NodeDataType::PositionSet2D(s) => s.get_new_children(),
            NodeDataType::PositionSet3D(s) => s.get_new_children(),
            NodeDataType::PositionPairSet2D(s) => s.get_new_children(),
            NodeDataType::PositionPairSet3D(s) => s.get_new_children(),
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
            NodeDataType::PositionSet2D(position_space) => position_space.is_child_valid(child_index),
            NodeDataType::PositionSet3D(position_space) => position_space.is_child_valid(child_index),
            NodeDataType::PositionPairSet2D(position_pair_set) => position_pair_set.is_child_valid(child_index),
            NodeDataType::PositionPairSet3D(position_pair_set) => position_pair_set.is_child_valid(child_index),
            NodeDataType::NumberSet(_)
            | NodeDataType::Build(_) 
            | NodeDataType::None 
            => panic!("Called is child valid on node that has no children"),
        }
    }

    pub fn get_number(&self, index: CollapseNodeKey) -> T {
        let node = self.nodes.get(index)
            .expect("Number Set by index not found");

        let v = match &node.data {
            NodeDataType::NumberSet(d) => d.value,
            _ => panic!("Template Node {:?} is not of Type Number Space", node.template_index)
        };

        v
    }

    pub fn get_position<V: Ve<T, D>, const D: usize>(&self, parent_index: CollapseNodeKey, child_index: CollapseChildKey) -> V {
        let parent = &self.nodes[parent_index];
       
        match D {
            2 => {
                let v = match &parent.data {
                    NodeDataType::PositionSet2D(d) => d.get_position(child_index),
                    _ => panic!("Template Node {:?} is not of Type Position Set 2D Set", parent.template_index)
                };

                // Safety V2 and V are the same Type bause D == 2
                unsafe { VUnion{ a: v }.b }
            }
            3 => {
                let v = match &parent.data {
                    NodeDataType::PositionSet3D(d) => d.get_position(child_index),
                    _ => panic!("Template Node {:?} is not of Type Position Set 3D Set", parent.template_index)
                };

                // Safety V3 and V are the same Type bause D == 3
                unsafe { VUnion{ a: v }.b }
            }
            _ => unreachable!()
        }
    }

    pub fn get_positions<V: Ve<T, D>, const D: usize>(
        &self,
        parent_index: CollapseNodeKey, 
    ) -> impl Iterator<Item = V> {

        let parent = &self.nodes[parent_index];
       
        match D {
            2 => {
                let v = match &parent.data {
                    NodeDataType::PositionSet2D(d) => d.get_positions(),
                    _ => panic!("Template Node {:?} is not of Type Position Set 2D Set", parent.template_index)
                };

                // Safety V2 and V are the same Type bause D == 2
                IM2::A(v.map(|v| unsafe { VUnion{ a: v }.b }))
            }
            3 => {
                let v = match &parent.data {
                    NodeDataType::PositionSet3D(d) => d.get_positions(),
                    _ => panic!("Template Node {:?} is not of Type Position Set 3D Set", parent.template_index)
                };

                // Safety V3 and V are the same Type bause D == 3
                IM2::B(v.map(|v| unsafe { VUnion{ a: v }.b }))
            }
            _ => unreachable!()
        }
    }

    pub fn get_position_pair<V: Ve<T, D>, const D: usize>(&self, parent_index: CollapseNodeKey, child_index: CollapseChildKey) -> (V, V) {
        let parent = &self.nodes[parent_index];
       
        match D {
            2 => {
                let v = match &parent.data {
                    NodeDataType::PositionPairSet2D(d) => d.get_position_pair(child_index),
                    _ => panic!("Template Node {:?} is not of Type Position Pair Set 2D Set", parent.template_index)
                };

                // Safety V2 and V are the same Type bause D == 2
                unsafe { PairUnion{ a: v }.b }
            }
            3 => {
                let v = match &parent.data {
                    NodeDataType::PositionPairSet3D(d) => d.get_position_pair(child_index),
                    _ => panic!("Template Node {:?} is not of Type Position Pair Set 3D Set", parent.template_index)
                };

                // Safety V3 and V are the same Type bause D == 3
                unsafe { PairUnion{ a: v }.b }
            }
            _ => unreachable!()
        }
    }

    pub fn get_position_pairs<V: Ve<T, D>, const D: usize>(&self, parent_index: CollapseNodeKey) -> impl Iterator<Item = (V, V)> {
        let parent = &self.nodes[parent_index];
       
        match D {
            2 => {
                let v = match &parent.data {
                    NodeDataType::PositionPairSet2D(d) => d.get_position_pairs(),
                    _ => panic!("Template Node {:?} is not of Type Position Pair Set 2D Set", parent.template_index)
                };

                // Safety V2 and V are the same Type bause D == 2
                IM2::A(v.map(|v| unsafe { PairUnion{ a: v }.b }))
            }
            3 => {
                let v = match &parent.data {
                    NodeDataType::PositionPairSet3D(d) => d.get_position_pairs(),
                    _ => panic!("Template Node {:?} is not of Type Position Pair Set 3D Set", parent.template_index)
                };

                // Safety V3 and V are the same Type bause D == 3
                IM2::B(v.map(|v| unsafe { PairUnion{ a: v }.b }))
            }
            _ => unreachable!()
        }
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
    ) -> (impl Iterator<Item = CollapseNodeKey>, bool) {
        if let Some((_, keys)) = get_value_data.depends.iter().find(|(i, _)| *i == template_index) {
            return (IM3::A(keys.iter().copied()), false);
        }

        if let Some((_, loop_path)) = get_value_data.depends_loop.iter().find(|(i, _)| *i == template_index) {
            let keys = self.get_depends_from_loop_path(get_value_data, loop_path);
            (IM3::B(keys.into_iter()), false)
        } else {
            (IM3::C(iter::empty()), true)
        }

    }

    fn get_child_index(
        &self, 
        index: CollapseNodeKey,
        get_value_data: GetValueData,
    ) -> Option<CollapseChildKey> {
        get_value_data.child_indexs.iter()
            .find(|(i, _)| *i == index)
            .map(|(_, ci)| *ci)
    }

      
    pub fn get_dependend_number(
        &self, 
        template_index: TemplateIndex,
        get_value_data: GetValueData,
    ) -> (impl Iterator<Item = T>, bool) {
        let (iter, r) = self.get_dependend_index_value_data(template_index, get_value_data);
        (iter.map(|i| self.get_number(i)), r)
    }
     
    pub fn get_dependend_position<V: Ve<T, D>, const D: usize>(
        &self, 
        template_index: TemplateIndex,
        get_value_data: GetValueData,
    ) -> (impl Iterator<Item = V>, bool) { 
        let (iter, r) = self.get_dependend_index_value_data(template_index, get_value_data);

        (iter.map(move |i| {
            let child_index = self.get_child_index(i, get_value_data);
            if let Some(child_index) = child_index {
                IM2::A(iter::once(self.get_position(i, child_index)))
            } else {
                IM2::B(self.get_positions(i))
            }
        }).flatten(), r)
    }

    pub fn get_dependend_position_pair<V: Ve<T, D>, const D: usize>(
        &self, 
        template_index: TemplateIndex,
        get_value_data: GetValueData,
    ) -> (impl Iterator<Item = (V, V)>, bool) { 
        let (iter, r) = self.get_dependend_index_value_data(template_index, get_value_data);
        
        (iter.map(move |i| {
            let child_index = self.get_child_index(i, get_value_data);
            if let Some(child_index) = child_index {
                IM2::A(iter::once(self.get_position_pair(i, child_index)))
            } else {
                IM2::B(self.get_position_pairs(i))
            }
        }).flatten(), r)
        
    }

    pub fn get_root_key(&self) -> CollapseNodeKey {
        self.nodes.keys().next().unwrap()
    }
}
