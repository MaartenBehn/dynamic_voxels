use octa_force::{anyhow::bail, OctaResult};

use crate::{model::{collapse::collapser::CollapseValueT, composer::output_state::OutputState, data_types::number_space::NumberSpaceValue}, util::default_types::T, volume::VolumeQureyPosValid};

#[derive(Debug, Clone, Default)]
pub struct NumberSet {
    pub value: T,
}

impl NumberSet { 
    pub fn update(&mut self, new_value: T) { 
        self.value = new_value;
    }
}

impl CollapseValueT for NumberSet {
    fn on_delete(&self,state: &mut OutputState) {
    }
}
