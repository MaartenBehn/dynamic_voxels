use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;

use std::{fmt::Debug, iter, marker::PhantomData, ops::RangeBounds};

use crate::{vec_csg_tree::tree::VecCSGNode, volume::Volume};

use super::{collapse::CollapseNode, relative_path::{self, RelativePathTree}, template::{NodeTemplateValue, TemplateTree}};

pub trait IT: Debug + Copy + Eq + Default {}
pub trait BU: Debug + Copy + Default {}

#[derive(Debug, Clone)]
pub struct ModelSynthesisBuilder<I: IT, V: Volume> {
    pub nodes: Vec<BuilderNode<I, V>>,
}

#[derive(Debug, Clone, Copy)]
pub enum BuilderAmmount<I: IT>{
    OneGlobal,
    OnePer(I),
    NPer(usize, I),
    DefinedBy(I),
}

#[derive(Debug, Clone)]
pub struct BuilderNode<I: IT, V: Volume> {
    pub identifier: I,
    pub value: NodeTemplateValue<V>,
    pub depends: Vec<I>,
    pub knows: Vec<I>,
    pub ammount: BuilderAmmount<I>,
}

#[derive(Debug, Clone)]
pub struct NodeBuilder<I: IT> {
    pub depends: Vec<I>,
    pub knows: Vec<I>,
    pub ammount: BuilderAmmount<I>,
}

impl<I: IT, V: Volume> ModelSynthesisBuilder<I, V> {
    pub fn new() -> ModelSynthesisBuilder<I, V> {
        ModelSynthesisBuilder {
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
            ammount: BuilderAmmount::OneGlobal
        };

        builder = b(builder);

        let node = BuilderNode {
            value: NodeTemplateValue::new_group(),
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
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
            ammount: BuilderAmmount::OneGlobal
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

        let node = BuilderNode {
            value: NodeTemplateValue::new_number_range(start_bound, end_bound),
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
        };
        
        self.nodes.push(node);
        
        self
    }

    pub fn grid(
        mut self,
        identifier: I,
        boundary: V,
        spacing: Vec3,
        b: fn(NodeBuilder<I>) -> NodeBuilder<I>
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal
        };

        builder = b(builder);

        let node = BuilderNode {
            value: NodeTemplateValue::new_grid(boundary, spacing),
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
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
            ammount: BuilderAmmount::OneGlobal
        };

        builder = b(builder);

        let node = BuilderNode {
            value: NodeTemplateValue::new_pos(),
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
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
            ammount: BuilderAmmount::OneGlobal
        };

        builder = b(builder);

        let node = BuilderNode {
            value: NodeTemplateValue::new_build(),
            identifier,
            depends: builder.depends, 
            knows: builder.knows, 
            ammount: builder.ammount,
        };

        self.nodes.push(node);

        self
    }

    
}

impl<I: IT> NodeBuilder<I> {

    pub fn ammount(mut self, ammount: BuilderAmmount<I>) -> Self {
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
}

impl<I: IT, V: Volume> ModelSynthesisBuilder<I, V> {
    pub fn get_node_index_by_identifier(&self, identifier: I) -> usize {
        self.nodes.iter().position(|n| n.identifier == identifier).expect(&format!("No Node with Identifier {:?} found.", identifier))
    }

    pub fn build_template(&self) -> TemplateTree<I, V> {
        TemplateTree::new_from_builder(self)
    }
}

impl<V: Volume> NodeTemplateValue<V> {
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
