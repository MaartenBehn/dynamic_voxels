use std::{iter, ops::RangeBounds};

use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;

use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};

use super::{builder::{BuilderAmmount, BuilderNode, ModelSynthesisBuilder, IT}, number_range::NumberRange, pos_set::PositionSet, relative_path::RelativePathTree};

pub type TemplateIndex = usize;
pub const TEMPLATE_INDEX_ROOT: TemplateIndex = 0;
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone)]
pub struct TemplateTree<I: IT, V: VolumeQureyPosValid, P: VolumeQureyPosValid2D> {
    pub nodes: Vec<TemplateNode<I, V, P>>,
    pub max_level: usize,
}

#[derive(Debug, Clone)]
pub enum NodeTemplateValue<V: VolumeQureyPosValid, P: VolumeQureyPosValid2D> {
    Groupe,
    NumberRangeHook,
    NumberRange(NumberRange),
    PosSetHook,
    PosSet(PositionSet<V, P>),
    BuildHook
}

#[derive(Debug, Clone)]
pub struct TemplateNode<I: IT, V: VolumeQureyPosValid, P: VolumeQureyPosValid2D> {
    pub identifier: I,
    pub index: TemplateIndex,
    pub value: NodeTemplateValue<V, P>,
    pub depends: Vec<TemplateIndex>,
    pub dependend: Vec<TemplateIndex>,
    pub knows: Vec<TemplateIndex>,
    pub level: usize,
    pub defines_n: Vec<TemplateAmmountN>,
    pub defines_by_value: Vec<TemplateAmmountValue>,
}

#[derive(Debug, Clone)]
pub struct TemplateAmmountN {
    pub ammount: usize,
    pub index: TemplateIndex,
    pub dependecy_tree: RelativePathTree,
}

#[derive(Debug, Clone)]
pub struct TemplateAmmountValue {
    pub index: TemplateIndex,
    pub dependecy_tree: RelativePathTree,
}

impl<I: IT, V: VolumeQureyPosValid, P: VolumeQureyPosValid2D> TemplateTree<I, V, P> {
    pub fn new_from_builder(builder: &ModelSynthesisBuilder<I, V, P>) -> TemplateTree<I, V, P> {
        let mut nodes = vec![TemplateNode { 
            identifier: I::default(),
            index: 0,
            value: NodeTemplateValue::Groupe {  }, 
            depends: vec![], 
            dependend: vec![], 
            knows: vec![], 
            level: 0,
            defines_n: vec![],
            defines_by_value: vec![],
        }];

        // Create the nodes
        for (i, builder_node) in builder.nodes.iter().enumerate() {
            let template_node = TemplateNode {
                identifier: builder_node.identifier,
                index: i + 1,
                value: builder_node.value.to_owned(),
                depends: vec![],
                dependend: vec![],
                knows: vec![],
                level: 0,
                defines_n: vec![],
                defines_by_value: vec![],
            };

            nodes.push(template_node);
        }

        // Set depends, knows and creates indecies 
        for (mut template_node_index, builder_node) in builder.nodes.iter().enumerate() {
            template_node_index += 1;

            let template_node = &nodes[template_node_index];

            let mut add_defines_n = |ammount: usize, parent_index: usize| -> usize {
                nodes[parent_index].defines_n.push(TemplateAmmountN{
                    ammount,
                    index: template_node_index,
                    dependecy_tree: RelativePathTree::default(),
                });
                parent_index
            };

            let parent_index = match builder_node.ammount {
                BuilderAmmount::OneGlobal => add_defines_n(1, 0), 
                BuilderAmmount::OnePer(i) => add_defines_n(1, builder.get_node_index_by_identifier(i) + 1),
                BuilderAmmount::NPer(n, i) => add_defines_n(n, builder.get_node_index_by_identifier(i) + 1),
                BuilderAmmount::DefinedBy(i) =>  {
                    let parent_index = builder.get_node_index_by_identifier(i) + 1;
                    nodes[parent_index].defines_by_value.push(TemplateAmmountValue{
                        index: template_node_index,
                        dependecy_tree: RelativePathTree::default(),
                    });
                    parent_index
                }
            };
            nodes[parent_index].dependend.push(template_node_index);
                
            let mut depends = vec![parent_index]; 
            for i in builder_node.depends.iter() {
                let depends_index = builder.get_node_index_by_identifier(*i) + 1;
                if !depends.contains(&depends_index) {
                    depends.push(depends_index);
                    nodes[depends_index].dependend.push(template_node_index);
                }
            }
            nodes[template_node_index].depends = depends;

            let mut knows = vec![];
            for i in builder_node.knows.iter() {
                let knows_index = builder.get_node_index_by_identifier(*i) + 1;
                if !knows.contains(&knows_index) {
                    knows.push(knows_index);
                }
            }
            nodes[template_node_index].knows = knows;
        }

        let mut tree = TemplateTree {
            nodes,
            max_level: 0,
        };

        // Set create paths und levels
        for i in 1..tree.nodes.len() {
            if tree.nodes[i].level == 0 {
                tree.set_level_of_node(i);
            }

            let index = tree.nodes[i].index;
            for j in 0..tree.nodes[i].defines_n.len() {
                let new_node = &tree.nodes[tree.nodes[i].defines_n[j].index];

                tree.nodes[i].defines_n[j].dependecy_tree = RelativePathTree::get_paths_to_other_dependcies_from_parent(
                    &tree, 
                    i,
                    &new_node.depends,
                    &new_node.knows);
            }

            for j in 0..tree.nodes[i].defines_by_value.len() {
                let new_node = &tree.nodes[tree.nodes[i].defines_by_value[j].index];

                tree.nodes[i].defines_by_value[j].dependecy_tree = RelativePathTree::get_paths_to_other_dependcies_from_parent(
                    &tree, 
                    i,
                    &new_node.depends,
                    &new_node.knows);
            }
        }

        tree
    } 

    fn set_level_of_node(&mut self, index: usize) -> usize {
        let node = &self.nodes[index];

        let mut max_level = 0;
        for index in iter::empty()
            .chain(node.depends.to_owned().iter())
            .chain(node.knows.to_owned().iter()) {

            let mut level = self.nodes[*index].level; 

            if level == 0 {
                level = self.set_level_of_node(*index);
            } 

            max_level = max_level.max(level);
        }

        let node_level = max_level + 1;
        self.nodes[index].level = node_level;
        self.max_level = self.max_level.max(node_level);

        node_level
    } 
}

impl<V: VolumeQureyPosValid, P: VolumeQureyPosValid2D> NodeTemplateValue<V, P> {
    pub fn new_group() -> NodeTemplateValue<V, P> {
        NodeTemplateValue::Groupe {}
    }

    pub fn new_number_range<R: RangeBounds<i32>>(range: R) -> NodeTemplateValue<V, P> {
        
        let min = match range.start_bound() {
            std::ops::Bound::Included(&num) => num,
            std::ops::Bound::Excluded(&num) => num + 1,
            std::ops::Bound::Unbounded => i32::MIN,
        };

        let max = match range.end_bound() {
            std::ops::Bound::Included(&num) => num + 1,
            std::ops::Bound::Excluded(&num) => num,
            std::ops::Bound::Unbounded => i32::MAX,
        };

        NodeTemplateValue::NumberRange(NumberRange::new(min, max))
    }

    pub fn new_position_set(set: PositionSet<V, P>) -> NodeTemplateValue<V, P> { 
        NodeTemplateValue::PosSet(set)
    }
 
    pub fn new_build() -> NodeTemplateValue<V, P> {
        NodeTemplateValue::BuildHook {}
    }
}
