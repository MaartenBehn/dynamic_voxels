use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;

use crate::csg_tree::tree::{CSGNode, CSGNodeData, CSGTree};

use std::{fmt::Debug, marker::PhantomData, ops::RangeBounds, usize};

use super::{collapse::Node, func_data::{BuildFuncData, PosCollapseFuncData}, volume::PossibleVolume};

pub type Identifier = usize;
pub const NODE_IDENTIFIER_NONE: Identifier = Identifier::MAX;

#[derive(Debug, Clone)]
pub struct WFCBuilder<U: Clone + Debug, B: Clone + Debug> {
    pub nodes: Vec<NodeTemplate<U, B>>,
    pub attributes: Vec<AttributeTemplate<U, B>>,
}

#[derive(Debug, Clone)]
pub struct NodeTemplate<U: Clone + Debug, B: Clone + Debug> {
    pub identifier: Option<Identifier>,
    pub name: String, 
    pub attributes: Vec<Identifier>,
    pub user_data: Option<U>,
    pub build: fn(d: BuildFuncData<U, B>) 
}

#[derive(Debug, Clone)]
pub enum AttributeTemplateValue<U: Clone + Debug, B: Clone + Debug> {
    NumberRange {
        min: i32,
        max: i32,
        defines: NumberRangeDefinesType,
        permutation: Permutation<DefaultBuildHasher>,
    }, 
    Pos {
        from_volume: Identifier,
        on_collapse_changes_volume: fn(d: PosCollapseFuncData<U, B>)
    },
}

#[derive(Debug, Clone)]
pub struct AttributeTemplate<U: Clone + Debug, B: Clone + Debug> {
    pub identifier: Identifier,
    pub value: AttributeTemplateValue<U, B>,
}

#[derive(Debug, Clone)]
pub struct WFCNodeBuilder<U: Clone + Debug, B: Clone + Debug> {
    pub identifier: Option<Identifier>,
    pub name: String, 
    pub attributes: Vec<AttributeTemplate<U, B>>,
    pub user_data: Option<U>,
    pub build: fn(d: BuildFuncData<U, B>) 
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
pub struct PosBuilder<U: Clone + Debug, B: Clone + Debug> {
    on_collapse_changes_volume: fn(d: PosCollapseFuncData<U, B>)
}


impl<U: Clone + Debug, B: Clone + Debug> WFCBuilder<U, B> {
    pub fn new() -> WFCBuilder<U, B> {
        WFCBuilder {
            nodes: vec![],
            attributes: vec![],
        }
    }

    pub fn node(
        mut self,
        build_node: fn(builder: WFCNodeBuilder<U, B>) -> WFCNodeBuilder<U, B>,
    ) -> WFCBuilder<U, B> {
        let mut builder = WFCNodeBuilder::new();
        builder = build_node(builder);

        builder.attributes.sort_by(|a, b| {
             
            let get_value = |x: &AttributeTemplate<U, B>| {
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
            build: builder.build,
        });

        self.attributes.append(&mut builder.attributes);
        self
    } 
}

impl<U: Clone + Debug, B: Clone + Debug> WFCNodeBuilder<U, B> {
    fn new() -> Self {
        WFCNodeBuilder {
            attributes: vec![],
            identifier: None,
            name: "".to_string(),
            user_data: None,
            build: |_| {},
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

    pub fn build(mut self, build: fn(d: BuildFuncData<U, B>)) -> Self {
        self.build = build;
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
                permutation: Permutation::new((end_bound - start_bound) as _, seed, DefaultBuildHasher::new())
            },
            identifier,
        };

        self.attributes.push(number_range);
 
        self
    }

    pub fn pos(
        mut self,
        identifier: Identifier,
        from_volume: Identifier,
        pos_options: fn(b: PosBuilder<U, B>) -> PosBuilder<U, B>,
    ) -> Self {

        let mut pos_builder = PosBuilder::new();
        pos_builder = pos_options(pos_builder);

        let seed = fastrand::u64(0..1000);
        let pos = AttributeTemplate {
            value: AttributeTemplateValue::Pos {
                from_volume,
                on_collapse_changes_volume: pos_builder.on_collapse_changes_volume,
            },
            identifier,
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

impl<U: Clone + Debug, B: Clone + Debug> PosBuilder<U, B> {
    pub fn new() -> Self {
        PosBuilder {
            on_collapse_changes_volume: |_| {},
        }
    }  

    pub fn on_collapse_changes_volume(mut self, func: fn(PosCollapseFuncData<U, B>)) -> Self {
        self.on_collapse_changes_volume = func;
        self
    }
}

impl <U: Clone + Debug, B: Clone + Debug> AttributeTemplateValue<U, B> {
    pub fn get_number_min(&self) -> i32 {
        match self {
            AttributeTemplateValue::NumberRange { min, .. } => *min,
            _ => unreachable!(),
        }
    }

    pub fn get_number_max(&self) -> i32 {
        match self {
            AttributeTemplateValue::NumberRange { max, .. } => *max,
            _ => unreachable!(),
        }
    }

    pub fn get_number_defines(&self) -> &NumberRangeDefinesType {
        match self {
            AttributeTemplateValue::NumberRange { defines, .. } => defines,
            _ => unreachable!(),
        }
    }  

    pub fn get_number_permutation(&self) -> &Permutation {
        match self {
            AttributeTemplateValue::NumberRange { permutation, .. } => permutation,
            _ => unreachable!(),
        }
    }  

}
