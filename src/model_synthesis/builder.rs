use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;


use std::{fmt::Debug, marker::PhantomData, ops::RangeBounds, usize};

use crate::vec_csg_tree::tree::VecCSGNode;

use super::{collapse::Node,  volume::PossibleVolume};

pub trait IT: Debug + Copy + Eq {}

#[derive(Debug, Clone)]
pub struct WFCBuilder<I: IT> {
    pub nodes: Vec<NodeTemplate<I>>,
}

#[derive(Debug, Clone)]
pub struct NodeTemplate<I: IT> {
    pub identifier: I,
    pub value: NodeTemplateValue,
    pub depends: Vec<I>,
    pub children: Vec<I>
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
pub struct GroupeBuilder<I: IT> {
    pub children: Vec<I>,
    pub depends: Vec<I>,
}

#[derive(Debug, Clone)]
pub struct NumberBuilder<I: IT> {
    pub children: Vec<I>,
    pub depends: Vec<I>,
}

#[derive(Debug, Clone)]
pub struct PosBuilder<I: IT> {
    pub children: Vec<I>,
    pub depends: Vec<I>,
}

#[derive(Debug, Clone)]
pub struct BuildHookBuilder<I: IT> {
    pub depends: Vec<I>,
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
        b: fn(GroupeBuilder<I>) -> GroupeBuilder<I>
    ) -> Self {
        let mut builder = GroupeBuilder {
            children: vec![],
            depends: vec![],
        };

        builder = b(builder);

        let node = NodeTemplate {
            value: NodeTemplateValue::Groupe {},
            identifier,
            depends: builder.depends,
            children: builder.children
        };

        self.nodes.push(node);
        
        self
    }

    pub fn number_range<R: RangeBounds<i32>>(
        mut self,
        identifier: I,
        range: R,
        b: fn(NumberBuilder<I>) -> NumberBuilder<I>
    ) -> Self {
        let mut builder = NumberBuilder {
            children: vec![],
            depends: vec![],
        };

        builder = b(builder);

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
            children: builder.children,
        };

        self.nodes.push(node);
        
        self
    }

    pub fn pos(
        mut self,
        identifier: I,
        b: fn(PosBuilder<I>) -> PosBuilder<I>
    ) -> Self {
        
        let mut builder = PosBuilder{
            children: vec![],
            depends: vec![],
        };
        builder = b(builder);

        let node = NodeTemplate {
            value: NodeTemplateValue::Pos {},
            identifier,
            depends: builder.depends,
            children: builder.children
        };

        self.nodes.push(node);

        self
    }

    pub fn volume(
        mut self, 
        identifier: I,
        volume: VecCSGNode,
        sample_distance: f32,
    ) -> Self {
        let node = NodeTemplate {
            value: NodeTemplateValue::Volume { 
                volume: PossibleVolume::new(volume, sample_distance) 
            },
            identifier,
            depends: vec![],
            children: vec![],
        };

        self.nodes.push(node);

        self
    }

    pub fn build(
        mut self, 
        identifier: I,
        b: fn(BuildHookBuilder<I>) -> BuildHookBuilder<I>
    ) -> Self {

        let mut builder = BuildHookBuilder{
            depends: vec![],
        };
        builder = b(builder);

        let node = NodeTemplate {
            value: NodeTemplateValue::BuildHook {  },
            identifier,
            depends: builder.depends,
            children: vec![],
        };

        self.nodes.push(node);

        self
    }
}

impl<I: IT> GroupeBuilder<I> {

    pub fn child(mut self, identifier: I) -> Self {
        self.children.push(identifier);
        self
    }

    pub fn depends(mut self, identifier: I) -> Self {
        self.depends.push(identifier);
        self
    } 
}

impl<I: IT> NumberBuilder<I> {

    pub fn child(mut self, identifier: I) -> Self {
        self.children.push(identifier);
        self
    }

    pub fn depends(mut self, identifier: I) -> Self {
        self.depends.push(identifier);
        self
    } 
}

impl<I: IT> PosBuilder<I> {
    
    pub fn child(mut self, identifier: I) -> Self {
        self.children.push(identifier);
        self
    }

    pub fn depends(mut self, identifier: I) -> Self {
        self.depends.push(identifier);
        self
    } 
}

impl<I: IT> BuildHookBuilder<I> {
    
    pub fn depends(mut self, identifier: I) -> Self {
        self.depends.push(identifier);
        self
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
