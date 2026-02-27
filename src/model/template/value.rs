use std::marker::PhantomData;

use egui_snarl::NodeId;
use octa_force::{egui::Vec2, glam::Vec3A};

use crate::{model::{collapse::template_changed::MatchValueData, data_types::{data_type::{T, V2, V3}, mesh::MeshTemplate, number::{Hook, NumberValue}, number_space::NumberSpaceValue, position::PositionValue, position_pair_set::PositionPairSetValue, position_set::PositionSetValue, position_space::PositionSpaceValue, volume::VolumeValue, voxels::VoxelValue}}, util::{number::Nu, vector::Ve}};

use super::{nodes, Template};

pub type ValueIndex = usize;
pub const VALUE_INDEX_NODE: usize = usize::MAX;
 

#[derive(Debug, Clone, Copy)]
pub enum TemplateValue {
    None,
    Number(NumberValue),
    NumberSet(NumberSpaceValue),
    Position2D(PositionValue<V2, 2>),
    Position3D(PositionValue<V3, 3>),
    PositionSet2D(PositionSetValue),
    PositionSet3D(PositionSetValue),
    PositionPairSet2D(PositionPairSetValue),
    PositionPairSet3D(PositionPairSetValue),
    PositionSpace2D(PositionSpaceValue),
    PositionSpace3D(PositionSpaceValue),
    Volume2D(VolumeValue),
    Volume3D(VolumeValue),
    Voxels(VoxelValue),
    Mesh(MeshTemplate),
}

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
    pub fn match_value(&self, other: &TemplateValue, match_value_data: MatchValueData) -> bool {
        match self {
            TemplateValue::None => match other {
                TemplateValue::None => true,
                _ => false,
            },
            TemplateValue::Number(number_template) => match other {
                TemplateValue::Number(other_numer_template) 
                    => number_template.match_value(other_numer_template, match_value_data),
                _ => false,
            },
            TemplateValue::NumberSet(number_space_template) => todo!(),
            TemplateValue::Position2D(position_template) => todo!(),
            TemplateValue::Position3D(position_template) => todo!(),
            TemplateValue::PositionSet2D(position_set_template) => todo!(),
            TemplateValue::PositionSet3D(position_set_template) => todo!(),
            TemplateValue::PositionPairSet2D(position_pair_set_template) => todo!(),
            TemplateValue::PositionPairSet3D(position_pair_set_template) => todo!(),
            TemplateValue::PositionSpace2D(position_space_template) => todo!(),
            TemplateValue::PositionSpace3D(position_space_template) => todo!(),
            TemplateValue::Volume2D(volume_template) => todo!(),
            TemplateValue::Volume3D(volume_template) => todo!(),
            TemplateValue::Voxels(voxel_template) => todo!(),
            TemplateValue::Mesh(mesh_template) => todo!(),
        }
    }
}


