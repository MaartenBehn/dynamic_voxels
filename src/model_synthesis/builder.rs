use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::Vec3;

use crate::csg_tree::tree::{CSGNode, CSGNodeData, CSGTree};

use std::{fmt::Debug, marker::PhantomData, ops::RangeBounds, usize};

use super::{collapse::Node,  volume::PossibleVolume};

pub trait IT: Debug + Copy + Eq {}

#[derive(Debug, Clone)]
pub struct WFCBuilder<I: IT> {
    pub nodes: Vec<NodeTemplate<I>>,
    pub attributes: Vec<AttributeTemplate<I>>,
}

#[derive(Debug, Clone)]
pub struct NodeTemplate<I: IT> {
    pub identifier: I,
    pub attributes: Vec<I>,
    pub build_hook: bool,
    pub children: Vec<I>,
}

#[derive(Debug, Clone)]
pub enum AttributeTemplateValue<I: IT> {
    NumberRange {
        min: i32,
        max: i32,
        defines: NumberRangeDefines<I>,
        permutation: Permutation<DefaultBuildHasher>,
    }, 
    Pos {
    },
    Volume {
        volume: PossibleVolume,
    }
}

#[derive(Debug, Clone)]
pub struct AttributeTemplate<I: IT> {
    pub identifier: I,
    pub value: AttributeTemplateValue<I>,
    pub build_hook: bool,
}

#[derive(Debug, Clone)]
pub struct WFCNodeBuilder<I: IT> {
    pub attributes: Vec<AttributeTemplate<I>>,
    pub build_hook: bool,
    pub children: Vec<I>,
}

#[derive(Debug, Clone)]
pub enum NumberRangeDefines<I: IT> {
    None,
    Amount { of_node: I },
}

impl<I: IT> WFCBuilder<I> {
    pub fn new() -> WFCBuilder<I> {
        WFCBuilder {
            nodes: vec![],
            attributes: vec![],
        }
    }

    pub fn node(
        mut self,
        identifier: I,
        build_node: fn(builder: WFCNodeBuilder<I>) -> WFCNodeBuilder<I>,
    ) -> WFCBuilder<I> {
        let mut builder = WFCNodeBuilder::new();
        builder = build_node(builder);

        builder.attributes.sort_by(|a, b| {
             
            let get_value = |x: &AttributeTemplate<I>| {
                match &x.value {
                    AttributeTemplateValue::NumberRange { defines, .. } => {
                        match defines {
                            NumberRangeDefines::None => 1,
                            NumberRangeDefines::Amount { .. } => 2,
                        } 
                    },
                    _ => 1,
                }
            };

            get_value(a).cmp(&get_value(b))
        });

        self.nodes.push(NodeTemplate {
            identifier,
            attributes: builder.attributes.iter()
                .map(|n| {
                    n.identifier
                })
                .collect(),
            build_hook: builder.build_hook,
            children: builder.children,
        });

        self.attributes.append(&mut builder.attributes);
        self
    } 
}

impl<I: IT> WFCNodeBuilder<I> {
    fn new() -> Self {
        WFCNodeBuilder {
            attributes: vec![],
            build_hook: false,
            children: vec![],
        }
    }

    pub fn use_build_hook(mut self) -> Self {
        self.build_hook = true;
        self
    }

    pub fn child(mut self, identifier: I) -> Self {
        self.children.push(identifier);
        self
    }
 
    pub fn number_range<R: RangeBounds<i32>>(
        mut self,
        identifier: I,
        range: R,
        defines: NumberRangeDefines<I>,
        build_hook: bool,
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


        let seed = fastrand::u64(0..1000);
        let number_range = AttributeTemplate {
            value: AttributeTemplateValue::NumberRange {
                defines,
                min: start_bound,
                max: end_bound,
                permutation: Permutation::new((end_bound - start_bound) as _, seed, DefaultBuildHasher::new())
            },
            identifier,
            build_hook,
        };

        self.attributes.push(number_range);
 
        self
    }

    pub fn pos(
        mut self,
        identifier: I,
        build_hook: bool,
    ) -> Self {
        let seed = fastrand::u64(0..1000);
        let pos = AttributeTemplate {
            value: AttributeTemplateValue::Pos {},
            identifier,
            build_hook,
        };

        self.attributes.push(pos);

        self
    }

    pub fn volume(
        mut self, 
        identifier: I,
        volume: CSGNode,
        sample_distance: f32,
        build_hook: bool,
    ) -> Self {
        let volume = AttributeTemplate {
            value: AttributeTemplateValue::Volume { 
                volume: PossibleVolume::new(volume, sample_distance) 
            },
            identifier,
            build_hook,
        };

        self.attributes.push(volume);

        self
    }
}

impl<I: IT> AttributeTemplateValue<I> {
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

    pub fn get_number_defines(&self) -> &NumberRangeDefines<I> {
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
