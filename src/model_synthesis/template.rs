use std::{iter, ops::RangeBounds};

use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;

use crate::{model_synthesis::relative_path::RelativePathTree, volume::VolumeQureyPosValid};

use super::{builder::{BuilderAmmount, BuilderNode, ModelSynthesisBuilder, IT}, pos_set::PositionSet};

pub type TemplateIndex = usize;
pub const TEMPLATE_INDEX_ROOT: TemplateIndex = 0;
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone)]
pub struct TemplateTree<I: IT, V: VolumeQureyPosValid> {
    pub nodes: Vec<TemplateNode<I, V>>,
    pub max_level: usize,
}

#[derive(Debug, Clone)]
pub enum NodeTemplateValue<V: VolumeQureyPosValid> {
    Groupe {},
    NumberRangeHook,
    NumberRange {
        min: i32,
        max: i32,
        permutation: Permutation<DefaultBuildHasher>,
    }, 
    PosSetHook,
    PosSet(PositionSet<V>),
    PosHook,
    Pos { value: Vec3 },
    BuildHook {}
}

#[derive(Debug, Clone)]
pub struct TemplateNode<I: IT, V: VolumeQureyPosValid> {
    pub identifier: I,
    pub index: TemplateIndex,
    pub value: NodeTemplateValue<V>,
    pub depends: Vec<TemplateIndex>,
    pub dependend: Vec<TemplateIndex>,
    pub knows: Vec<TemplateIndex>,
    pub level: usize,
    pub defines_ammount: Vec<TemplateAmmount>,
}

#[derive(Debug, Clone)]
pub struct TemplateAmmount {
    pub typ: TemplateAmmountType,
    pub index: TemplateIndex,
    pub dependecy_tree: RelativePathTree,
}

#[derive(Debug, Clone)]
pub enum TemplateAmmountType{
    N(usize),
    Value,
}

impl<I: IT, V: VolumeQureyPosValid> TemplateTree<I, V> {
    pub fn new_from_builder(builder: &ModelSynthesisBuilder<I, V>) -> TemplateTree<I, V> {
        let mut nodes = vec![TemplateNode { 
            identifier: I::default(),
            index: 0,
            value: NodeTemplateValue::Groupe {  }, 
            depends: vec![], 
            dependend: vec![], 
            knows: vec![], 
            level: 0,
            defines_ammount: vec![],
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
                defines_ammount: vec![],
            };

            nodes.push(template_node);
        }

        // Set depends, knows and creates indecies 
        for (mut template_node_index, builder_node) in builder.nodes.iter().enumerate() {
            template_node_index += 1;

            let template_node = &nodes[template_node_index];

            let (typ, parent_node_index) = Self::get_ammount_type_and_defines_index(builder, builder_node, &nodes);
            nodes[parent_node_index].defines_ammount.push(TemplateAmmount{
                typ,
                index: template_node_index,
                dependecy_tree: RelativePathTree::default(),
            });
            nodes[parent_node_index].dependend.push(template_node_index);
                
            let mut depends = vec![parent_node_index]; 
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
            for j in 0..tree.nodes[i].defines_ammount.len() {
                let new_node = &tree.nodes[tree.nodes[i].defines_ammount[j].index];

                tree.nodes[i].defines_ammount[j].dependecy_tree = RelativePathTree::get_paths_to_other_dependcies_from_parent(
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

    fn get_ammount_type_and_defines_index(builder: &ModelSynthesisBuilder<I, V>, builder_node: &BuilderNode<I, V>, nodes: &[TemplateNode<I, V>]) -> (TemplateAmmountType, usize) {
        match builder_node.ammount {
            BuilderAmmount::OneGlobal => (TemplateAmmountType::N(1), 0), 
            BuilderAmmount::OnePer(i) => (TemplateAmmountType::N(1), builder.get_node_index_by_identifier(i) + 1),
            BuilderAmmount::NPer(n, i) => (TemplateAmmountType::N(n), builder.get_node_index_by_identifier(i) + 1),
            BuilderAmmount::DefinedBy(i) =>  (TemplateAmmountType::Value, builder.get_node_index_by_identifier(i) + 1),
        }

    }
}

impl<V: VolumeQureyPosValid> NodeTemplateValue<V> {
    pub fn new_group() -> NodeTemplateValue<V> {
        NodeTemplateValue::Groupe {}
    }

    pub fn new_number_range<R: RangeBounds<i32>>(range: R) -> NodeTemplateValue<V> {
        
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

        let seed = fastrand::u64(0..1000);
        NodeTemplateValue::NumberRange {
            min,
            max,
            permutation: Permutation::new((max - min) as _, seed, DefaultBuildHasher::new())
        }
    }

    pub fn new_position_set(set: PositionSet<V>) -> NodeTemplateValue<V> { 
        NodeTemplateValue::PosSet(set)
    }

    pub fn new_pos( value: Vec3 ) -> NodeTemplateValue<V> {
        NodeTemplateValue::Pos { value }
    }

    /*
    pub fn new_grid(mut boundary: V, spacing: Vec3) -> NodeTemplateValue<V> {
        let aabb = boundary.get_aabb();

        let mut points = vec![];
        let mut point = aabb.min;
        while point.x <= aabb.max.x {
            while point.y <= aabb.max.y {
                while point.z <= aabb.max.z {
                    if boundary.is_position_valid_vec3(point) {
                        points.push(point);
                    }  
                } 
                point.y += spacing.y;
                point.z = aabb.min.z;
            } 
            point.x += spacing.x;
            point.y = aabb.min.y;
        } 

        NodeTemplateValue::Grid { boundary, spacing, points }
    }
    */

    pub fn new_build() -> NodeTemplateValue<V> {
        NodeTemplateValue::BuildHook {}
    }
}
