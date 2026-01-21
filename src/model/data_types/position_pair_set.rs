use std::cmp::Ordering;

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::{iproduct, Either, Itertools};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::{GetNewChildrenData, GetValueData}, collapser::{CollapseChildKey, CollapseNodeKey, Collapser}}, composer::{ModelComposer, nodes::ComposeNodeType}, data_types::data_type::T, template::{self, Template, TemplateIndex, update::MakeTemplateData, value::TemplateValue}}, util::{iter_merger::IM2, math_config::MC, vector::Ve}};

use crate::util::vector;
use crate::util::math_config;

use super::{data_type::ComposeDataType, number::{Hook, NumberTemplate, ValueIndexNumber}, position_space::ValueIndexPositionSpace};

pub type ValueIndexPositionSet = usize;
pub type ValueIndexPositionSet2D = usize;
pub type ValueIndexPositionSet3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionPairSetTemplate {
    ByDistance((ValueIndexPositionSpace, ValueIndexNumber)),
}

impl PositionPairSetTemplate {
    pub fn get_value<V: Ve<T, D>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser,
        template: &Template
    ) -> (Vec<(V, V)>, bool) {
        match self {
            PositionPairSetTemplate::ByDistance((space, distance)) => {
                let (set, r_0) = template.get_position_space_value(*space)
                    .get_value::<V, D>(get_value_data, collapser, template);

                let (distance, r_1) = template.get_number_value(*distance)
                    .get_value(get_value_data, collapser, template);

                let max_dist = distance.into_iter().fold(0.0, |a, b| if (a < b) { b } else { a });
                let dist_squared = max_dist * max_dist;

                let set_2 = set.clone();

                let pairs = iproduct!(set, set_2)
                    .filter(move |(a, b)| matches!(a.cmp(*b), Ordering::Less) && (*a - *b).length_squared() < dist_squared );

                (pairs.collect(), r_0 || r_1)
            },
        }
    }
}
