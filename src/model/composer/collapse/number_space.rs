use octa_force::{anyhow::bail, OctaResult};

use crate::{model::composer::{build::BS, number_space::NumberSpaceTemplate, template::TemplateIndex}, util::{number::Nu, vector::Ve}, volume::VolumeQureyPosValid};

use super::collapser::{CollapseNode, CollapseNodeKey, Collapser};


#[derive(Debug, Clone)]
pub struct NumberSpace<T: Nu> {
    pub values: Vec<T>,
    pub value: T,
}

impl<T: Nu> NumberSpace<T> {
    pub fn from_template<V2: Ve<T, 2>, V3: Ve<T, 3>, B: BS<V2, V3, T>>(
        space_template: &NumberSpaceTemplate<T>, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> Self {
        match space_template {
            NumberSpaceTemplate::NumberRange { min, max, step } => {
                let min = min.get_value(depends, collapser);
                let max = max.get_value(depends, collapser);
                let step = step.get_value(depends, collapser);

                let mut values = vec![];
                let mut i = min;
                while i <= max {
                    values.push(i);
                    i += step;
                }

                Self {
                    values,
                    value: T::ZERO,
                }
            },
        }

    }

    pub fn update(&mut self) -> OctaResult<()> {
        if self.values.is_empty() {
            bail!("Number Range empty");
        }

        let i = fastrand::usize(0..self.values.len());
        self.value = self.values[i];

        Ok(())
    }
}
