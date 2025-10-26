use crate::{model::{composer::build::BS, data_types::{number::NumberTemplate, number_space::NumberSpaceTemplate, position::PositionTemplate, position_set::PositionSetTemplate, position_space::PositionSpaceTemplate, volume::VolumeTemplate}}, util::{number::Nu, vector::Ve}};

use super::ComposeTemplate;

pub type ValueIndex = usize;
pub const VALUE_INDEX_NODE: usize = usize::MAX;

#[derive(Debug, Clone, Copy)]
pub enum ComposeTemplateValue<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    None,
    Number(NumberTemplate<T>),
    NumberSpace(NumberSpaceTemplate),
    Position2D(PositionTemplate<V2, T, 2>),
    Position3D(PositionTemplate<V3, T, 3>),
    PositionSet(PositionSetTemplate),
    PositionSpace(PositionSpaceTemplate),
    Volume(VolumeTemplate),
    Build(B::TemplateValue)
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeTemplate<V2, V3, T, B> {
    pub fn has_value(&self, value_index: ValueIndex) -> bool {
        if self.values.len() <= value_index {
            return false;
        } 

        if matches!(self.values[value_index], ComposeTemplateValue::None) {
            false
        } else {
            true
        }
    }

    pub fn set_value(&mut self, value_index: ValueIndex, value: ComposeTemplateValue<V2, V3, T, B>) {
        if self.values.len() <= value_index {
            self.values.resize(value_index + 1, ComposeTemplateValue::None);
        }

        self.values[value_index] = value;
    } 
}
