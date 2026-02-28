use std::marker::PhantomData;

use egui_snarl::NodeId;
use octa_force::{egui::Vec2, glam::Vec3A};

use crate::{model::{collapse::template_changed::MatchValueData, data_types::{data_type::{T, TemplateValue, V2, V3}, mesh::MeshTemplate, number::{Hook, NumberValue}, number_space::NumberSpaceValue, position::PositionValue, position_pair_set::PositionPairSetValue, position_set::PositionSetValue, position_space::PositionSpaceValue, volume::VolumeValue, voxels::VoxelValue}}, util::{number::Nu, vector::Ve}};

use super::{nodes, Template};

pub type ValueIndex = usize;
pub const VALUE_INDEX_NODE: usize = usize::MAX;


pub union PositionTemplateUnion<'a, VA: Ve<T, DA>, VB: Ve<T, DB>, const DA: usize, const DB: usize> {
    a: &'a PositionValue<VA, DA>,
    b: &'a PositionValue<VB, DB>,
}

impl Template {
    pub fn get_number_value(&self, value_index: ValueIndex) -> &NumberValue {
        match &self.values[value_index] {
            TemplateValue::Number(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_number_space_value(&self, value_index: ValueIndex) -> &NumberSpaceValue {
        match &self.values[value_index] {
            TemplateValue::NumberSet(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_value<V: Ve<T, D>, const D: usize>(&self, value_index: ValueIndex) -> &PositionValue<V, D> {
        match &self.values[value_index] {
            TemplateValue::Position2D(v) => {
                debug_assert!(D == 2);
                unsafe { PositionTemplateUnion{ a: v }.b }
            },
            TemplateValue::Position3D(v) => {
                debug_assert!(D == 3);
                unsafe { PositionTemplateUnion{ a: v }.b }
            },
            _ => unreachable!()
        }
    }

    pub fn get_position2d_value(&self, value_index: ValueIndex) -> &PositionValue<V2, 2> {
        match &self.values[value_index] {
            TemplateValue::Position2D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position3d_value(&self, value_index: ValueIndex) -> &PositionValue<V3, 3> {
        match &self.values[value_index] {
            TemplateValue::Position3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_set_value(&self, value_index: ValueIndex) -> &PositionSetValue {
        match &self.values[value_index] {
            TemplateValue::PositionSet2D(v)
            | TemplateValue::PositionSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_pair_set_value(&self, value_index: ValueIndex) -> &PositionPairSetValue {
        match &self.values[value_index] {
            TemplateValue::PositionPairSet2D(v)
            | TemplateValue::PositionPairSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_space_value(&self, value_index: ValueIndex) -> &PositionSpaceValue {
        match &self.values[value_index] {
            TemplateValue::PositionSpace2D(v)
            | TemplateValue::PositionSpace3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_volume_value(&self, value_index: ValueIndex) -> &VolumeValue {
        match &self.values[value_index] {
            TemplateValue::Volume2D(v)
            | TemplateValue::Volume3D(v) => v,
            _ => unreachable!()
        }
    }


    pub fn get_number_value_mut(&mut self, value_index: ValueIndex) -> &mut NumberValue {
        match &mut self.values[value_index] {
            TemplateValue::Number(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_number_space_value_mut(&mut self, value_index: ValueIndex) -> &mut NumberSpaceValue {
        match &mut self.values[value_index] {
            TemplateValue::NumberSet(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position2d_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionValue<V2, 2> {
        match &mut self.values[value_index] {
            TemplateValue::Position2D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position3d_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionValue<V3, 3> {
        match &mut self.values[value_index] {
            TemplateValue::Position3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_set_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionSetValue {
        match &mut self.values[value_index] {
            TemplateValue::PositionSet2D(v)
            | TemplateValue::PositionSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_volume_value_mut(&mut self, value_index: ValueIndex) -> &mut VolumeValue {
        match &mut self.values[value_index] {
            TemplateValue::Volume2D(v)
            | TemplateValue::Volume3D(v) => v,
            _ => unreachable!()
        }
    }
}

impl TemplateValue {
    pub fn match_template_value(&self, other: &TemplateValue, data: MatchValueData) -> bool {
        match self {
            TemplateValue::None => match other {
                TemplateValue::None => true,
                _ => false,
            },
            TemplateValue::PositionSet2D(a) => match other {
                TemplateValue::PositionSet2D(b) 
                    => a.match_value(b, data),
                _ => false,
            },
            TemplateValue::PositionSet3D(a) => match other {
                TemplateValue::PositionSet3D(b) 
                    => a.match_value(b, data),
                _ => false,
            },
            TemplateValue::PositionPairSet2D(a) => match other {
                TemplateValue::PositionPairSet2D(b) 
                    => a.match_value(b, data),
                _ => false,
            },
            TemplateValue::PositionPairSet3D(a) => match other {
                TemplateValue::PositionPairSet3D(b) 
                    => a.match_value(b, data),
                _ => false,
            },
            TemplateValue::Voxels(a) => match other {
                TemplateValue::Voxels(b) 
                    => a.match_value(b, data),
                _ => false,
            },
            TemplateValue::Mesh(a) => match other {
                TemplateValue::Mesh(b) 
                    => a.match_value(b, data),
                _ => false,
            },
            _ => unreachable!()
        }
    }
}


