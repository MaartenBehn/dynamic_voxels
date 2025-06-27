use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;

use std::{fmt::Debug, iter, marker::PhantomData, ops::RangeBounds};

use crate::{vec_csg_tree::tree::VecCSGNode, volume::VolumeQureyPosValid};

use super::{collapse::CollapseNode, pos_set::PositionSet, relative_path::{self, RelativePathTree}, template::{NodeTemplateValue, TemplateTree}};

pub trait IT: Debug + Copy + Eq + Default {}
pub trait BU: Debug + Copy + Default {}

#[derive(Debug, Clone)]
pub struct ModelSynthesisBuilder<I: IT, V: VolumeQureyPosValid> {
    pub nodes: Vec<BuilderNode<I, V>>,
}

#[derive(Debug, Clone, Copy)]
pub enum BuilderAmmount<I: IT>{
    OneGlobal,
    OnePer(I),
    NPer(usize, I),
    DefinedBy(I),
}

#[derive(Debug, Clone, Copy)]
pub enum BuilderValue<T>{
    Const(T),
    Hook,
}

#[derive(Debug, Clone)]
pub struct BuilderNode<I: IT, V: VolumeQureyPosValid> {
    pub identifier: I,
    pub value: NodeTemplateValue<V>,
    pub depends: Vec<I>,
    pub knows: Vec<I>,
    pub ammount: BuilderAmmount<I>,
}

#[derive(Debug, Clone)]
pub struct NodeBuilder<I: IT, T> {
    pub depends: Vec<I>,
    pub knows: Vec<I>,
    pub ammount: BuilderAmmount<I>,
    pub value: BuilderValue<T> 
}

impl<I: IT, V: VolumeQureyPosValid> ModelSynthesisBuilder<I, V> {
    pub fn new() -> ModelSynthesisBuilder<I, V> {
        ModelSynthesisBuilder {
            nodes: vec![],
        }
    }

    pub fn groupe(
        mut self,
        identifier: I,
        b: fn(NodeBuilder<I, ()>) -> NodeBuilder<I, ()>
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Const(())
        };

        builder = b(builder);

        assert!(
            matches!(builder.value, BuilderValue::Const(())), 
            "Groupe Value only supports: Const(())"
        );

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

    pub fn number_range<R: RangeBounds<i32>, F: FnOnce(NodeBuilder<I, R>) -> NodeBuilder<I, R>>(
        mut self,
        identifier: I,
        b: F, 
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Hook
        };

        builder = b(builder);

        let node = BuilderNode {
            value: match builder.value {
                BuilderValue::Const(r) => NodeTemplateValue::new_number_range(r),
                BuilderValue::Hook => NodeTemplateValue::NumberRangeHook,
            },
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
        };

        self.nodes.push(node);
        
        self
    }

    pub fn position_set<F: FnOnce(NodeBuilder<I, PositionSet<V>>) -> NodeBuilder<I, PositionSet<V>>>(
        mut self,
        identifier: I,
        b: F, 
    ) -> Self {

        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Hook
        };

        builder = b(builder);

        let node = BuilderNode {
            value: match builder.value {
                BuilderValue::Const(s) => NodeTemplateValue::new_position_set(s),
                BuilderValue::Hook => NodeTemplateValue::PosHook,
            },
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
        };

        self.nodes.push(node);

        self
    }

    pub fn pos<F: FnOnce(NodeBuilder<I, Vec3>) -> NodeBuilder<I, Vec3>>(
        mut self,
        identifier: I,
        b: F,
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Hook
        };

        builder = b(builder);

        let node = BuilderNode {
            value: match builder.value {
                BuilderValue::Const(v) =>  NodeTemplateValue::new_pos(v),
                BuilderValue::Hook => NodeTemplateValue::PosHook,
            },             
            identifier,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
        };

        self.nodes.push(node);

        self
    }
 
    pub fn build<F: FnOnce(NodeBuilder<I, ()>) -> NodeBuilder<I, ()>>(
        mut self, 
        identifier: I,
        b: F, 
    ) -> Self {
        let mut builder = NodeBuilder {
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Hook,
        };

        builder = b(builder);

        let node = BuilderNode {
            value: match builder.value {
                BuilderValue::Hook => NodeTemplateValue::new_build(),
                _ => panic!("Build Value only supports: Hook"),
            },            
            identifier,
            depends: builder.depends, 
            knows: builder.knows, 
            ammount: builder.ammount,
        };

        self.nodes.push(node);

        self
    }

    
}

impl<I: IT, T> NodeBuilder<I, T> {

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

    pub fn value(mut self, v: BuilderValue<T>) -> Self {
        self.value = v;
        self
    }
}

impl<I: IT, V: VolumeQureyPosValid> ModelSynthesisBuilder<I, V> {
    pub fn get_node_index_by_identifier(&self, identifier: I) -> usize {
        self.nodes.iter()
            .position(|n| n.identifier == identifier)
            .expect(&format!("No Node with Identifier {:?} found.", identifier))
    }

    pub fn build_template(&self) -> TemplateTree<I, V> {
        TemplateTree::new_from_builder(self)
    }
}

impl<V: VolumeQureyPosValid> NodeTemplateValue<V> {
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
