use std::marker::PhantomData;

use egui_snarl::NodeId;

use crate::{model::{composer::build::BS, data_types::{number::NumberTemplate, number_space::NumberSpaceTemplate, position::PositionTemplate, position_pair_set::PositionPairSetTemplate, position_set::PositionSetTemplate, position_space::PositionSpaceTemplate, volume::VolumeTemplate}}, util::{number::Nu, vector::Ve}};

use super::{nodes, ComposeTemplate};

pub type ValueIndex = usize;
pub const VALUE_INDEX_NODE: usize = usize::MAX;

#[derive(Debug, Clone, Copy)]
pub enum ComposeTemplateValue<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    None,
    Number(NumberTemplate<T>),
    NumberSet(NumberSpaceTemplate),
    Position2D(PositionTemplate<V2, T, 2>),
    Position3D(PositionTemplate<V3, T, 3>),
    PositionSet2D(PositionSetTemplate),
    PositionSet3D(PositionSetTemplate),
    PositionPairSet2D(PositionPairSetTemplate),
    PositionPairSet3D(PositionPairSetTemplate),
    PositionSpace2D(PositionSpaceTemplate),
    PositionSpace3D(PositionSpaceTemplate),
    Volume2D(VolumeTemplate),
    Volume3D(VolumeTemplate),
    Build(B::TemplateValue)
}

pub union PositionTemplateUnion<'a, VA: Ve<T, DA>, VB: Ve<T, DB>, T: Nu, const DA: usize, const DB: usize> {
    a: &'a PositionTemplate<VA, T, DA>,
    b: &'a PositionTemplate<VB, T, DB>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeTemplate<V2, V3, T, B> {
    pub fn get_number_value(&self, value_index: ValueIndex) -> &NumberTemplate<T> {
        match &self.values[value_index] {
            ComposeTemplateValue::Number(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_number_space_value(&self, value_index: ValueIndex) -> &NumberSpaceTemplate {
        match &self.values[value_index] {
            ComposeTemplateValue::NumberSet(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_value<V: Ve<T, D>, const D: usize>(&self, value_index: ValueIndex) -> &PositionTemplate<V, T, D> {
        match &self.values[value_index] {
            ComposeTemplateValue::Position2D(v) => {
                debug_assert!(D == 2);
                unsafe { PositionTemplateUnion{ a: v }.b }
            },
            ComposeTemplateValue::Position3D(v) => {
                debug_assert!(D == 3);
                unsafe { PositionTemplateUnion{ a: v }.b }
            },
            _ => unreachable!()
        }
    }

    pub fn get_position2d_value(&self, value_index: ValueIndex) -> &PositionTemplate<V2, T, 2> {
        match &self.values[value_index] {
            ComposeTemplateValue::Position2D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position3d_value(&self, value_index: ValueIndex) -> &PositionTemplate<V3, T, 3> {
        match &self.values[value_index] {
            ComposeTemplateValue::Position3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_set_value(&self, value_index: ValueIndex) -> &PositionSetTemplate {
        match &self.values[value_index] {
            ComposeTemplateValue::PositionSet2D(v)
            | ComposeTemplateValue::PositionSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_pair_set_value(&self, value_index: ValueIndex) -> &PositionPairSetTemplate {
        match &self.values[value_index] {
            ComposeTemplateValue::PositionPairSet2D(v)
            | ComposeTemplateValue::PositionPairSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_space_value(&self, value_index: ValueIndex) -> &PositionSpaceTemplate {
        match &self.values[value_index] {
            ComposeTemplateValue::PositionSpace2D(v)
            | ComposeTemplateValue::PositionSpace3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_volume_value(&self, value_index: ValueIndex) -> &VolumeTemplate {
        match &self.values[value_index] {
            ComposeTemplateValue::Volume2D(v)
            | ComposeTemplateValue::Volume3D(v) => v,
            _ => unreachable!()
        }
    }



    pub fn get_number_value_mut(&mut self, value_index: ValueIndex) -> &mut NumberTemplate<T> {
        match &mut self.values[value_index] {
            ComposeTemplateValue::Number(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_number_space_value_mut(&mut self, value_index: ValueIndex) -> &mut NumberSpaceTemplate {
        match &mut self.values[value_index] {
            ComposeTemplateValue::NumberSet(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position2d_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionTemplate<V2, T, 2> {
        match &mut self.values[value_index] {
            ComposeTemplateValue::Position2D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position3d_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionTemplate<V3, T, 3> {
        match &mut self.values[value_index] {
            ComposeTemplateValue::Position3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_set_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionSetTemplate {
        match &mut self.values[value_index] {
            ComposeTemplateValue::PositionSet2D(v)
            | ComposeTemplateValue::PositionSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_volume_value_mut(&mut self, value_index: ValueIndex) -> &mut VolumeTemplate {
        match &mut self.values[value_index] {
            ComposeTemplateValue::Volume2D(v)
            | ComposeTemplateValue::Volume3D(v) => v,
            _ => unreachable!()
        }
    }
}


