use std::marker::PhantomData;

use egui_snarl::NodeId;
use octa_force::{egui::Vec2, glam::Vec3A};

use crate::{model::{collapse::template_changed::MatchValueData, data_types::{data_type::{T, V2, V3}, mesh::MeshTemplate, number::{Hook, NumberTemplate}, number_space::NumberSpaceTemplate, position::PositionTemplate, position_pair_set::PositionPairSetTemplate, position_set::PositionSetTemplate, position_space::PositionSpaceTemplate, volume::VolumeTemplate, voxels::VoxelTemplate}}, util::{number::Nu, vector::Ve}};

use super::{nodes, Template};

pub type ValueIndex = usize;
pub const VALUE_INDEX_NODE: usize = usize::MAX;
 

#[derive(Debug, Clone, Copy)]
pub enum TemplateValue {
    None,
    Number(NumberTemplate),
    NumberSet(NumberSpaceTemplate),
    Position2D(PositionTemplate<V2, 2>),
    Position3D(PositionTemplate<V3, 3>),
    PositionSet2D(PositionSetTemplate),
    PositionSet3D(PositionSetTemplate),
    PositionPairSet2D(PositionPairSetTemplate),
    PositionPairSet3D(PositionPairSetTemplate),
    PositionSpace2D(PositionSpaceTemplate),
    PositionSpace3D(PositionSpaceTemplate),
    Volume2D(VolumeTemplate),
    Volume3D(VolumeTemplate),
    Voxels(VoxelTemplate),
    Mesh(MeshTemplate),
}

pub union PositionTemplateUnion<'a, VA: Ve<T, DA>, VB: Ve<T, DB>, const DA: usize, const DB: usize> {
    a: &'a PositionTemplate<VA, DA>,
    b: &'a PositionTemplate<VB, DB>,
}

impl Template {
    pub fn get_number_value(&self, value_index: ValueIndex) -> &NumberTemplate {
        match &self.values[value_index] {
            TemplateValue::Number(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_number_space_value(&self, value_index: ValueIndex) -> &NumberSpaceTemplate {
        match &self.values[value_index] {
            TemplateValue::NumberSet(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_value<V: Ve<T, D>, const D: usize>(&self, value_index: ValueIndex) -> &PositionTemplate<V, D> {
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

    pub fn get_position2d_value(&self, value_index: ValueIndex) -> &PositionTemplate<V2, 2> {
        match &self.values[value_index] {
            TemplateValue::Position2D(v) => v,
            _ => {
                dbg!(value_index); 
                dbg!(&self.values[value_index]); 
                unreachable!();
            }
        }
    }

    pub fn get_position3d_value(&self, value_index: ValueIndex) -> &PositionTemplate<V3, 3> {
        match &self.values[value_index] {
            TemplateValue::Position3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_set_value(&self, value_index: ValueIndex) -> &PositionSetTemplate {
        match &self.values[value_index] {
            TemplateValue::PositionSet2D(v)
            | TemplateValue::PositionSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_pair_set_value(&self, value_index: ValueIndex) -> &PositionPairSetTemplate {
        match &self.values[value_index] {
            TemplateValue::PositionPairSet2D(v)
            | TemplateValue::PositionPairSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_space_value(&self, value_index: ValueIndex) -> &PositionSpaceTemplate {
        match &self.values[value_index] {
            TemplateValue::PositionSpace2D(v)
            | TemplateValue::PositionSpace3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_volume_value(&self, value_index: ValueIndex) -> &VolumeTemplate {
        match &self.values[value_index] {
            TemplateValue::Volume2D(v)
            | TemplateValue::Volume3D(v) => v,
            _ => unreachable!()
        }
    }


    pub fn get_number_value_mut(&mut self, value_index: ValueIndex) -> &mut NumberTemplate {
        match &mut self.values[value_index] {
            TemplateValue::Number(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_number_space_value_mut(&mut self, value_index: ValueIndex) -> &mut NumberSpaceTemplate {
        match &mut self.values[value_index] {
            TemplateValue::NumberSet(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position2d_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionTemplate<V2, 2> {
        match &mut self.values[value_index] {
            TemplateValue::Position2D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position3d_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionTemplate<V3, 3> {
        match &mut self.values[value_index] {
            TemplateValue::Position3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_position_set_value_mut(&mut self, value_index: ValueIndex) -> &mut PositionSetTemplate {
        match &mut self.values[value_index] {
            TemplateValue::PositionSet2D(v)
            | TemplateValue::PositionSet3D(v) => v,
            _ => unreachable!()
        }
    }

    pub fn get_volume_value_mut(&mut self, value_index: ValueIndex) -> &mut VolumeTemplate {
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


