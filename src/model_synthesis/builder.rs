use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;

use std::{fmt::Debug, iter, marker::PhantomData, ops::RangeBounds, usize};

use crate::vec_csg_tree::tree::VecCSGNode;

use super::{collapse::Node,  volume::PossibleVolume};

pub trait IT: Debug + Copy + Eq {}
pub trait BU: Debug + Copy + Default {}

#[derive(Debug, Clone)]
pub struct WFCBuilder<I: IT> {
    pub nodes: Vec<NodeTemplate<I>>,
}

#[derive(Debug, Clone)]
pub enum Ammount<I: IT>{
    OneGlobal,
    OnePer(I),
    NPer(usize, I),
    DefinedBy(I),
}

#[derive(Debug, Clone)]
pub struct NodeTemplate<I: IT> {
    pub identifier: I,
    pub value: NodeTemplateValue,
    pub depends: Vec<I>,
    pub knows: Vec<I>,
    pub ammount: Ammount<I>,
    pub level: usize,
}

#[derive(Debug, Clone)]
pub enum NodeTemplateValue {
    Groupe {},
    NumberRange {
        min: i32,
        max: i32,
        permutation: Permutation<DefaultBuildHasher>,
    }, 
    Pos {},
    Volume {
        volume: PossibleVolume,
    },
    BuildHook {}
}

#[derive(Debug, Clone)]
pub struct NodeBuilder<I: IT> {
    pub depends: Vec<I>,
    pub knows: Vec<I>,
    pub ammount: Ammount<I>,
}

impl<I: IT> WFCBuilder<I> {
    pub fn new() -> WFCBuilder<I> {
        WFCBuilder {
            nodes: vec![],
        }
    }

    pub fn groupe(
        mut self,
        identifier: I,
        b: fn(NodeBuilder<I>) -> NodeBuilder<I>
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: Ammount::OneGlobal
        };

        builder = b(builder);
        builder.add_ammount_to_depends();

        let node = NodeTemplate {
            value: NodeTemplateValue::Groupe {},
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
            level: 0
        };
        self.nodes.push(node);
        
        self
    }

    pub fn number_range<R: RangeBounds<i32>>(
        mut self,
        identifier: I,
        range: R,
        b: fn(NodeBuilder<I>) -> NodeBuilder<I>
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: Ammount::OneGlobal
        };

        builder = b(builder);
        builder.add_ammount_to_depends();

        let start_bound = match range.start_bound() {
            std::ops::Bound::Included(&num) => num,
            std::ops::Bound::Excluded(&num) => num + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end_bound = match range.end_bound() {
            std::ops::Bound::Included(&num) => num + 1,
            std::ops::Bound::Excluded(&num) => num,
            std::ops::Bound::Unbounded => panic!("Range can not be unbounded"),
        };


        let seed = fastrand::u64(0..1000);
        let node = NodeTemplate {
            value: NodeTemplateValue::NumberRange {
                min: start_bound,
                max: end_bound,
                permutation: Permutation::new((end_bound - start_bound) as _, seed, DefaultBuildHasher::new())
            },
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
            level: 0
        };

        self.nodes.push(node);
        
        self
    }

    pub fn pos(
        mut self,
        identifier: I,
        b: fn(NodeBuilder<I>) -> NodeBuilder<I>
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: Ammount::OneGlobal
        };

        builder = b(builder);
        builder.add_ammount_to_depends();

        let node = NodeTemplate {
            value: NodeTemplateValue::Pos {},
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
            level: 0
        };

        self.nodes.push(node);

        self
    }

    pub fn volume(
        mut self, 
        identifier: I,
        volume: VecCSGNode,
        sample_distance: f32,
        b: fn(NodeBuilder<I>) -> NodeBuilder<I>
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: Ammount::OneGlobal
        };

        builder = b(builder);
        builder.add_ammount_to_depends();

        let node = NodeTemplate {
            value: NodeTemplateValue::Volume { 
                volume: PossibleVolume::new(volume, sample_distance) 
            },
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
            level: 0
        };

        self.nodes.push(node);

        self
    }

    pub fn build(
        mut self, 
        identifier: I,
        b: fn(NodeBuilder<I>) -> NodeBuilder<I>
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: Ammount::OneGlobal
        };

        builder = b(builder);
        builder.add_ammount_to_depends();

        let node = NodeTemplate {
            value: NodeTemplateValue::BuildHook {  },
            identifier,
            depends: builder.depends, 
            knows: builder.knows, 
            ammount: builder.ammount,
            level: 0
        };

        self.nodes.push(node);

        self
    }
}

impl<I: IT> NodeBuilder<I> {

    pub fn ammount(mut self, ammount: Ammount<I>) -> Self {
        self.ammount = ammount;
        self
    }

    pub fn depends(mut self, identifier: I) -> Self {
        self.depends.push(identifier);
        self
    }

    pub fn knows(mut self, identifier: I) -> Self {
        self.knows.push(identifier);
        self
    }

    fn add_ammount_to_depends(&mut self) {
        match self.ammount {
            Ammount::OneGlobal => {},
            Ammount::NPer(i)
            | Ammount::DefinedBy(i) => {
                self.depends.push(value);
            },
        }
    }
}


impl<I: IT> WFCBuilder<I> {
    pub fn get_node_index_by_identifier(&self, identifier: I) -> usize {
        self.nodes.iter().position(|n| n.identifier == identifier).expect(&format!("No Node with Identifier {:?} found.", identifier))
    }

    pub fn set_levels(&mut self) {
        
        for i in 0..self.nodes.len() {
            if self.nodes[i].level != 0 {
                continue;
            }

            self.set_level_of_node(i);
        }
    }

    fn set_level_of_node(&mut self, index: usize) -> usize {
        let node = &self.nodes[index];

        let mut max_level = 0;
        for identifier in iter::empty()
            .chain(node.depends.to_owned().iter())
            .chain(node.knows.to_owned().iter()) {
            let i = self.get_node_index_by_identifier(*identifier);

            let mut level = self.nodes[i].level; 

            if level == 0 {
                level = self.set_level_of_node(i);
            } 

            max_level = max_level.max(level);
        }

        let node_level = max_level + 1;
        self.nodes[index].level = node_level;

        node_level
    }

    
}

impl NodeTemplateValue {
    pub fn get_number_min(&self) -> i32 {
        match self {
            NodeTemplateValue::NumberRange { min, .. } => *min,
            _ => unreachable!(),
        }
    }

    pub fn get_number_max(&self) -> i32 {
        match self {
            NodeTemplateValue::NumberRange { max, .. } => *max,
            _ => unreachable!(),
        }
    }
    
    pub fn get_number_permutation(&self) -> &Permutation {
        match self {
            NodeTemplateValue::NumberRange { permutation, .. } => permutation,
            _ => unreachable!(),
        }
    } 
}
