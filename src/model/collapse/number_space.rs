use octa_force::{anyhow::bail, OctaResult};

use crate::{model::{composer::{build::BS, template::TemplateIndex}, data_types::number_space::NumberSpaceTemplate}, util::{number::Nu, vector::Ve}, volume::VolumeQureyPosValid};

use super::{add_nodes::GetValueData, collapser::{CollapseNode, CollapseNodeKey, Collapser}};


#[derive(Debug, Clone)]
pub struct NumberSpace<T: Nu> {
    pub values: Vec<T>,
    pub value: T,
}

impl<T: Nu> NumberSpace<T> {
    pub fn from_template<V2: Ve<T, 2>, V3: Ve<T, 3>, B: BS<V2, V3, T>>(
        space_template: &NumberSpaceTemplate<V2, V3, T>,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (Self, bool) {
        match space_template {
            NumberSpaceTemplate::NumberRange { min, max, step } => {
                let (min, r_0) = min.get_value(get_value_data, collapser);
                let (max, r_1) = max.get_value(get_value_data, collapser);
                let (step, r_2) = step.get_value(get_value_data, collapser);

                let mut values = vec![];
                let mut i = min;
                while i <= max {
                    values.push(i);
                    i += step;
                }

                (Self {
                    values,
                    value: T::ZERO,
                }, r_0 || r_1 || r_2)
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
