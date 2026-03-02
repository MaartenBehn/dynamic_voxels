use std::marker::PhantomData;

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::{iproduct, Either, Itertools};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::{GetNewChildrenData, GetValueData}, collapser::{CollapseChildKey, CollapseNodeKey, CollapseValueT, Collapser}, template_changed::MatchValueData}, composer::{ModelComposer, nodes::ComposeNode, output_state::OutputState}, template::{self, Template, TemplateIndex}}, util::{default_types::T, iter_merger::IM2, math_config::MC, number::Nu, vector::Ve}};

use crate::util::vector;
use crate::util::math_config;

use super::{data_type::ComposeDataType, number::{Hook, NumberValue, ValueIndexNumber}, position_space::ValueIndexPositionSpace};

pub type ValueIndexPositionSet = usize;
pub type ValueIndexPositionSet2D = usize;
pub type ValueIndexPositionSet3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionSetValue {
    All(ValueIndexPositionSpace),
}
 
impl PositionSetValue {
    pub fn match_value(
        &self, 
        other: &PositionSetValue,
        data: MatchValueData
    ) -> bool {

        match self {
            PositionSetValue::All(ps1) => match other {
                PositionSetValue::All(ps2) => data.match_two_position_space(*ps1, *ps2),
                _ => false
            },
        }
    }

    pub fn get_value<V: Ve<T, D>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser,
    ) -> (Vec<V>, bool) {
        match self {
            PositionSetValue::All(space) => {
                collapser.template.get_position_space_value(*space)
                    .get_value(get_value_data, collapser)
            },
        }
    }
}


#[derive(Debug, Clone, Default)]
pub struct PositionSet<V: Ve<T, D>, T: Nu, const D: usize> {
    positions: SlotMap<CollapseChildKey, V>,
    new_children: Vec<CollapseChildKey>,
    deleted_children: Vec<CollapseChildKey>,
    p: PhantomData<T>,
}

impl<V: Ve<T, D>, T: Nu, const D: usize> PositionSet<V, T, D> { 
    pub fn get_position(&self, index: CollapseChildKey) -> V {
        self.positions[index]    
    }

    pub fn get_positions(&self) -> impl Iterator<Item = V> {

        self.positions.values().into_iter().map(|v| *v)
    }
 
    pub fn is_child_valid(&self, index: CollapseChildKey) -> bool {
        self.positions.contains_key(index)    
    }

    pub fn update(
        &mut self,
        mut new_positions: Vec<V>,
    ) {
        self.positions.retain(|_, p| {
            if let Some(i) = new_positions.iter().position(|t| *t == *p) {
                new_positions.swap_remove(i);
                true
            } else {
                false
            }
        });

        let new_children = new_positions.iter()
            .map(|p| self.positions.insert(*p))
            .collect_vec();

        self.new_children = new_children;
    }

    pub fn get_new_children(&self) -> &[CollapseChildKey] {
        &self.new_children
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> CollapseValueT for PositionSet<V, T, D> {
    fn on_delete(&self, state: &mut OutputState) {
    }
}
