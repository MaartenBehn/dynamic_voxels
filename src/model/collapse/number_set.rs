use octa_force::{anyhow::bail, OctaResult};

use crate::{model::data_types::{data_type::T, number_space::NumberSpaceTemplate}, volume::VolumeQureyPosValid};

use super::{add_nodes::GetValueData, collapser::{CollapseNode, CollapseNodeKey, Collapser}};

#[derive(Debug, Clone, Default)]
pub struct NumberSet {
    pub value: T,
}

impl NumberSet { 
    pub fn update(&mut self, new_value: T) { 
        self.value = new_value;
    }
}
