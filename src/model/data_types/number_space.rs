use std::iter;

use egui_snarl::OutPinId;
use itertools::{iproduct, Itertools};
use octa_force::{log::debug, OctaResult};
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::ComposeNodeType, ModelComposer}, template::{self, update::MakeTemplateData, value::{ComposeTemplateValue, ValueIndex}, ComposeTemplate}}, util::{number::Nu, vector::Ve}};

use super::{number::NumberTemplate};

pub type ValueIndexNumberSpace = usize;

#[derive(Debug, Clone, Copy)]
pub enum NumberSpaceTemplate {
    NumberRange {
        min: ValueIndexNumberSpace,
        max: ValueIndexNumberSpace,
        step: ValueIndexNumberSpace,
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_number_space(
        &self, 
        pin: OutPinId, 
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> ValueIndexNumberSpace {
        if let Some(value_index) = data.value_per_node_id.get_value(pin.node) {
            return value_index;
        }

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        let value = match &node.t {
            ComposeNodeType::NumberRange => {
                let min = self.make_number(node, 0, data);
                let max = self.make_number(node, 1, data);
                let step = self.make_number(node, 2, data);
                NumberSpaceTemplate::NumberRange { min, max, step }
            },
            _ => unreachable!(),
        };

        data.set_value(pin.node, ComposeTemplateValue::NumberSpace(value))
    }
}

impl NumberSpaceTemplate { 
    pub fn get_value<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> (impl Iterator<Item = T> + use<B, V2, V3, T>, bool) {
        match &self {
            NumberSpaceTemplate::NumberRange { min, max, step } => {
                let (min, r_0) =  template
                    .get_number_value(*min)
                    .get_value(get_value_data, collapser, template);

                let (max, r_1) =  template
                    .get_number_value(*max)
                    .get_value(get_value_data, collapser, template);

                let (step, r_2) =  template
                    .get_number_value(*step)
                    .get_value(get_value_data, collapser, template);

                let v = iproduct!(min, max, step)
                    .map(|(min, max, step)| {
                        let options = ((max - min) / step).to_usize();
                        let i = fastrand::usize(0..options);
                        let v = min + step * T::from_usize(i);
                        v
                    });

                (v, r_0 || r_1 || r_2)
            },
        }
    }
}


