use std::{cmp::Ordering, marker::PhantomData};

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::{iproduct, Either, Itertools};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::{GetNewChildrenData, GetValueData}, collapser::{CollapseChildKey, CollapseNodeKey, CollapseValueT, Collapser}, template_changed::MatchValueData}, composer::{ModelComposer, output_state::OutputState}, data_types::data_type::CollapseValue, template::{self, Template, TemplateIndex}}, util::{default_types::T, iter_merger::IM2, math_config::MC, number::Nu, vector::Ve}};

use crate::util::vector;
use crate::util::math_config;

use super::{data_type::ComposeDataType, number::{Hook, NumberValue, ValueIndexNumber}, position_space::ValueIndexPositionSpace};

pub type ValueIndexPositionSet = usize;
pub type ValueIndexPositionSet2D = usize;
pub type ValueIndexPositionSet3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionPairSetValue {
    ByDistance((ValueIndexPositionSpace, ValueIndexNumber)),
}

impl PositionPairSetValue {
    pub fn match_value(
        &self, 
        other: &PositionPairSetValue,
        data: MatchValueData
    ) -> bool {

        match self {
            PositionPairSetValue::ByDistance((ps1, n1)) => match other {
                PositionPairSetValue::ByDistance((ps2, n2)) => {
                    data.template.get_position_space_value(*ps1).match_value(
                            data.other_template.get_position_space_value(*ps2), 
                            data)

                    && data.match_two_numbers(*n1, *n2)
                },
                _ => false
            },
        }
    }

    pub fn get_value<V: Ve<T, D>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser,
    ) -> (Vec<(V, V)>, bool) {
        match self {
            PositionPairSetValue::ByDistance((space, distance)) => {
                let (set, r_0) = collapser.template.get_position_space_value(*space)
                    .get_value::<V, D>(get_value_data, collapser);

                let (distance, r_1) = collapser.template.get_number_value(*distance)
                    .get_value(get_value_data, collapser);

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

#[derive(Debug, Clone, Default)]
pub struct PositionPairSet<V: Ve<T, D>, T: Nu, const D: usize> {
    positions: SlotMap<CollapseChildKey, (V, V)>,
    new_children: Vec<CollapseChildKey>,
    p: PhantomData<T>,
}

impl<V: Ve<T, D>,  T: Nu, const D: usize> PositionPairSet<V, T, D> { 
    pub fn get_position_pair(&self, index: CollapseChildKey) -> (V, V) {
        self.positions[index]    
    }

    pub fn get_position_pairs(&self) -> impl Iterator<Item = (V, V)> {
        self.positions.values().into_iter().map(|v| *v)
    }
    
    pub fn is_child_valid(&self, index: CollapseChildKey) -> bool {
        self.positions.contains_key(index)    
    }

    pub fn update(
        &mut self,
        mut new_pairs: Vec<(V, V)>,
    ) {
        self.positions.retain(|_, p| {
            if let Some(i) = new_pairs.iter().position(|t| *t == *p) {
                new_pairs.swap_remove(i);
                true
            } else {
                false
            }
        });

        let new_children = new_pairs.iter()
            .map(|p| self.positions.insert(*p))
            .collect_vec();

        self.new_children = new_children;
    }

    pub fn get_new_children(&self) -> &[CollapseChildKey] {
        &self.new_children
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> CollapseValueT for PositionPairSet<V, T, D> {
    fn on_delete(&self, state: &mut OutputState) {
    }
}
