use octa_force::{anyhow::bail, OctaResult};

use crate::volume::VolumeQureyPosValid;

use super::{collapse::{CollapseNodeKey, CollapseOperation, Collapser}, template::TemplateTree};

#[derive(Debug, Clone)]
pub struct NumberRange {
    pub values: Vec<i32>,
    pub value: i32,
}

impl NumberRange {
    pub fn new(min: i32, max: i32) -> Self {
        Self {
            values: (min..=max).collect(),
            value: 0,
        }
    }

    pub fn next_value(&mut self) -> OctaResult<()> {
        if self.values.is_empty() {
            bail!("Number Range empty");
        }

        let i = fastrand::usize(0..self.values.len());
        self.value = self.values[i];

        Ok(())
    }
}

 
