use octa_force::{anyhow::bail, OctaResult};

use crate::{model::composer::{number_space::NumberSpace, template::TemplateIndex}, volume::VolumeQureyPosValid};

use super::collapser::{CollapseNode, CollapseNodeKey, Collapser};


#[derive(Debug, Clone)]
pub struct NumberSet {
    pub values: Vec<i32>,
    pub value: i32,
}

impl NumberSet {
    pub fn from_space(
        space: &NumberSpace, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser
    ) -> Self {
        match space {
            NumberSpace::NumberRange { min, max } => {
                let min = min.get_value(depends, collapser);
                let max = max.get_value(depends, collapser);

                Self {
                    values: (min..=max).collect(),
                    value: 0,
                }
            },
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
