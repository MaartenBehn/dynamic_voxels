use octa_force::{anyhow::bail, OctaResult};

use crate::{model::{composer::{build::BS, template::TemplateIndex}, data_types::number_space::NumberSpaceTemplate}, util::{number::Nu, vector::Ve}, volume::VolumeQureyPosValid};

use super::{add_nodes::GetValueData, collapser::{CollapseNode, CollapseNodeKey, Collapser}};

#[derive(Debug, Clone, Default)]
pub struct NumberSpace<T: Nu> {
    pub value: T,
}

impl<T: Nu> NumberSpace<T> { 
    pub fn update(&mut self, new_value: T) { 
        self.value = new_value;
    }
}
