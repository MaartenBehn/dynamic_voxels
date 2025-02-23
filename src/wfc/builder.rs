use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;

use crate::cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree};

use std::{fmt::Debug, marker::PhantomData, ops::RangeBounds, usize};

use super::collapse::{Attribute, CollapseFuncData, Node};

pub type Identifier = usize;
pub const NodeIdentifierNone: Identifier = Identifier::MAX;

#[derive(Debug, Clone)]
pub struct WFCBuilder<U: Clone + Debug> {
    pub nodes: Vec<NodeTemplate<U>>,
    pub attributes: Vec<AttributeTemplate<U>>,
}

#[derive(Debug, Clone)]
pub struct NodeTemplate<U: Clone + Debug> {
    pub identifier: Option<Identifier>,
    pub name: String, 
    pub attributes: Vec<Identifier>,
    pub user_data: Option<U>,
}

#[derive(Debug, Clone)]
pub enum AttributeTemplateValue<U: Clone + Debug> {
    NumberRange {
        min: i32,
        max: i32,
        defines: NumberRangeDefinesType,
    }, 
    Pos {
        collapse: fn(d: CollapseFuncData<U>) -> Option<Vec3>,
    },
}

#[derive(Debug, Clone)]
pub struct AttributeTemplate<U: Clone + Debug> {
    pub identifier: Identifier,
    pub permutation: Permutation<DefaultBuildHasher>,
    pub value: AttributeTemplateValue<U>,
}

#[derive(Debug, Clone)]
pub struct WFCNodeBuilder<U: Clone + Debug> {
    pub identifier: Option<Identifier>,
    pub name: String, 
    pub attributes: Vec<AttributeTemplate<U>>,
    pub user_data: Option<U>,
}

#[derive(Debug, Clone)]
pub enum NumberRangeDefinesType {
    None,
    Amount { of_node: Identifier },
}

#[derive(Debug, Clone)]
pub struct NumberRangeBuilder {
    pub defines: NumberRangeDefinesType,
}

#[derive(Debug, Clone)]
pub struct PosBuilder {
}


impl<U: Clone + Debug> WFCBuilder<U> {
    pub fn new() -> WFCBuilder<U> {
        WFCBuilder {
            nodes: vec![],
            attributes: vec![],
        }
    }

    pub fn node(
        mut self,
        build_node: fn(builder: WFCNodeBuilder<U>) -> WFCNodeBuilder<U>,
    ) -> WFCBuilder<U> {
        let mut builder = WFCNodeBuilder::new();
        builder = build_node(builder);

        builder.attributes.sort_by(|a, b| {
             
            let get_value = |x: &AttributeTemplate<U>| {
                match &x.value {
                    AttributeTemplateValue::NumberRange { defines, .. } => {
                        match defines {
                            NumberRangeDefinesType::None => 1,
                            NumberRangeDefinesType::Amount { .. } => 2,
                        } 
                    },
                    _ => 1,
                }
            };

            get_value(a).cmp(&get_value(b))
        });

        self.nodes.push(NodeTemplate {
            identifier: builder.identifier,
            name: builder.name,
            attributes: builder.attributes.iter()
                .map(|n| {
                    n.identifier
                })
                .collect(),
            user_data: builder.user_data,
        });

        self.attributes.append(&mut builder.attributes);
        self
    } 
}

impl<U: Clone + Debug> WFCNodeBuilder<U> {
    fn new() -> Self {
        WFCNodeBuilder {
            attributes: vec![],
            identifier: None,
            name: "".to_string(),
            user_data: None,
        }
    }

    pub fn identifier(mut self, identifier: Identifier) -> Self {
        self.identifier = Some(identifier);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn user_data(mut self, user_data: U) -> Self {
        self.user_data = Some(user_data);
        self
    }


    pub fn number_range<R: RangeBounds<i32>>(
        mut self,
        identifier: Identifier,
        range: R,
        number_set_options: fn(b: NumberRangeBuilder) -> NumberRangeBuilder,
    ) -> Self {
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

        let mut number_set_builder = NumberRangeBuilder::new();
        number_set_builder = number_set_options(number_set_builder);

        let seed = fastrand::u64(0..1000);
        let number_range = AttributeTemplate {
            value: AttributeTemplateValue::NumberRange {
                defines: number_set_builder.defines,
                min: start_bound,
                max: end_bound,
            },
            identifier,
            permutation: Permutation::new((end_bound - start_bound) as _, seed, DefaultBuildHasher::new())
        };

        self.attributes.push(number_range);
 
        self
    }

    pub fn pos(
        mut self,
        identifier: Identifier,
        num_collapses: usize,
        collapse: fn(d: CollapseFuncData<U>) -> Option<Vec3>,
        pos_options: fn(b: PosBuilder) -> PosBuilder,
    ) -> Self {

        let mut pos_builder = PosBuilder::new();
        pos_builder = pos_options(pos_builder);

        let seed = fastrand::u64(0..1000);
        let pos = AttributeTemplate {
            value: AttributeTemplateValue::Pos { 
                collapse  
            },
            identifier,
            permutation: Permutation::new(num_collapses as _, seed, DefaultBuildHasher::new())
        };

        self.attributes.push(pos);

        self
    } 
}

impl NumberRangeBuilder {
    pub fn new() -> Self {
        NumberRangeBuilder {
            defines: NumberRangeDefinesType::None,
        }
    }

    pub fn defines(mut self, defines: NumberRangeDefinesType) -> Self {
        self.defines = defines;
        self
    }
}

impl PosBuilder {
    pub fn new() -> Self {
        PosBuilder {
        }
    }  
}
