use octa_force::{glam::Vec3, log::error};

use std::{fmt::Debug, iter, marker::PhantomData, ops::RangeBounds};

use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};

use super::{collapse::CollapseNode, pos_set::PositionSet, relative_path::{self, RelativePathTree}, template::{NodeTemplateValue, TemplateTree}, traits::ModelGenerationTypes};


#[derive(Debug, Clone)]
pub struct ModelSynthesisBuilder<T: ModelGenerationTypes>{
    pub nodes: Vec<BuilderNode<T>>,
}

#[derive(Debug, Clone, Copy)]
pub enum BuilderAmmount<T: ModelGenerationTypes>{
    OneGlobal,
    OnePer(T::Identifier),
    NPer(usize, T::Identifier),
    DefinedBy(T::Identifier),
}

#[derive(Debug, Clone, Copy)]
pub enum BuilderValue<V>{
    Const(V),
    Hook,
}

#[derive(Debug, Clone)]
pub struct BuilderNode<T: ModelGenerationTypes> {
    pub identifier: T::Identifier,
    pub value: NodeTemplateValue<T>,
    pub restricts: Vec<T::Identifier>,
    pub depends: Vec<T::Identifier>,
    pub knows: Vec<T::Identifier>,
    pub ammount: BuilderAmmount<T>,
}

#[derive(Debug, Clone)]
pub struct NodeBuilder<T: ModelGenerationTypes, V> {
    pub restricts: Vec<T::Identifier>,
    pub depends: Vec<T::Identifier>,
    pub knows: Vec<T::Identifier>,
    pub ammount: BuilderAmmount<T>,
    pub value: BuilderValue<V> 
}

impl<T: ModelGenerationTypes> ModelSynthesisBuilder<T> {
    pub fn new() -> ModelSynthesisBuilder<T> {
        ModelSynthesisBuilder {
            nodes: vec![],
        }
    }

    pub fn groupe(
        mut self,
        identifier: T::Identifier,
        b: fn(NodeBuilder<T, ()>) -> NodeBuilder<T, ()>
    ) -> Self {
        let mut builder = NodeBuilder {
            restricts: vec![],
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Const(())
        };

        builder = b(builder);
        builder.check_valid(identifier);

        assert!(
            matches!(builder.value, BuilderValue::Const(())), 
            "Groupe Value only supports: Const(())"
        );

        let node = BuilderNode {
            value: NodeTemplateValue::new_group(),
            identifier,
            restricts: builder.restricts,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
        };
        self.nodes.push(node);
        
        self
    }

    pub fn number_range<R: RangeBounds<i32>, F: FnOnce(NodeBuilder<T, R>) -> NodeBuilder<T, R>>(
        mut self,
        identifier: T::Identifier,
        b: F, 
    ) -> Self {
        let mut builder = NodeBuilder {
            restricts: vec![],
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Hook
        };

        builder = b(builder);
        builder.check_valid(identifier);

        let node = BuilderNode {
            value: match builder.value {
                BuilderValue::Const(r) => NodeTemplateValue::new_number_range(r),
                BuilderValue::Hook => NodeTemplateValue::NumberRangeHook,
            },
            identifier,
            restricts: builder.restricts,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
        };

        self.nodes.push(node);
        
        self
    }

    pub fn position_set<F: FnOnce(NodeBuilder<T, PositionSet<T>>) -> NodeBuilder<T, PositionSet<T>>>(
        mut self,
        identifier: T::Identifier,
        b: F, 
    ) -> Self {

        let mut builder = NodeBuilder {
            restricts: vec![],
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Hook
        };

        builder = b(builder);
        builder.check_valid(identifier);

        let node = BuilderNode {
            value: match builder.value {
                BuilderValue::Const(s) => NodeTemplateValue::new_position_set(s),
                BuilderValue::Hook => NodeTemplateValue::PosSetHook,
            },
            identifier,
            restricts: builder.restricts,
            depends: builder.depends,
            knows: builder.knows,
            ammount: builder.ammount,
        };

        self.nodes.push(node);

        self
    }
 
    pub fn build<F: FnOnce(NodeBuilder<T, ()>) -> NodeBuilder<T, ()>>(
        mut self, 
        identifier: T::Identifier,
        b: F, 
    ) -> Self {
        let mut builder = NodeBuilder {
            restricts: vec![],
            depends: vec![],
            knows: vec![],
            ammount: BuilderAmmount::OneGlobal,
            value: BuilderValue::Hook,
        };

        builder = b(builder);
        builder.check_valid(identifier);

        let node = BuilderNode {
            value: match builder.value {
                BuilderValue::Hook => NodeTemplateValue::new_build(),
                _ => panic!("Build Value only supports: Hook"),
            },            
            identifier,
            restricts: builder.restricts,
            depends: builder.depends, 
            knows: builder.knows, 
            ammount: builder.ammount,
        };

        self.nodes.push(node);

        self
    }

    
}

impl<T: ModelGenerationTypes, V> NodeBuilder<T, V> {

    pub fn ammount(mut self, ammount: BuilderAmmount<T>) -> Self {
        self.ammount = ammount;
        self
    }

    pub fn depends(mut self, identifier: T::Identifier) -> Self {
        self.depends.push(identifier);
        self
    }

    pub fn knows(mut self, identifier: T::Identifier) -> Self {
        self.knows.push(identifier);
        self
    }

    pub fn value(mut self, v: BuilderValue<V>) -> Self {
        self.value = v;
        self
    }

    pub fn restricts(mut self, identifier: T::Identifier) -> Self {
        self.restricts.push(identifier);
        self
    }

    fn check_valid(&mut self, identifier: T::Identifier) {
        for i in (0..self.knows.len()).rev() {
            if self.restricts.contains(&self.knows[i]) {
                error!("{:?} - {:?} found in restricts and knows. Only use restricts -> removimg from knows", identifier, self.knows[i]);
                self.knows.swap_remove(i);
                continue;
            }

            if self.depends.contains(&self.knows[i]) {
                error!("{:?} - {:?} found in depends and knows. Only use depends -> removimg from knows", identifier, self.knows[i]);
                self.knows.swap_remove(i);
                continue;
            }
        }

        for i in (0..self.depends.len()).rev() {
            if self.restricts.contains(&self.depends[i]) {
                error!("{:?} - {:?} found in restricts and depends. Only use restricts -> removimg from depends", identifier, self.depends[i]);
                self.depends.swap_remove(i);
                continue;
            }
        }
    }
}

impl<T: ModelGenerationTypes> ModelSynthesisBuilder<T> {
    pub fn get_node_index_by_identifier(&self, identifier: T::Identifier) -> usize {
        self.nodes.iter()
            .position(|n| n.identifier == identifier)
            .expect(&format!("No Node with Identifier {:?} found.", identifier))
    }

    pub fn build_template(&self) -> TemplateTree<T> {
        TemplateTree::new_from_builder(self)
    }
}
