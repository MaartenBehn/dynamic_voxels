use std::iter;

use egui_snarl::OutPinId;
use itertools::Itertools;
use octa_force::{log::debug, OctaResult};
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::ComposeNodeType, ModelComposer}, template::{update::MakeTemplateData, value::ComposeTemplateValue}}, util::{number::Nu, vector::Ve}};

use super::number::NumberTemplate;

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
        let value_index = pin.node.0;
        if data.template.has_value(value_index) {
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

        data.template.set_value(value_index, ComposeTemplateValue::NumberSpace(value));
        return value_index;
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> NumberSpaceTemplate { 
    pub fn get_value<B: BS<V2, V3, T>>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (impl Iterator<Item = T> + use<B, V2, V3, T>, bool) {
        match &self {
            NumberSpaceTemplate::NumberRange { min, max, step } => {
                let (min, r_0) = min.get_value(get_value_data, collapser);
                let (max, r_1) = max.get_value(get_value_data, collapser);
                let (step, r_2) = step.get_value(get_value_data, collapser);

                let v = min.into_iter()
                    .cartesian_product(max)
                    .cartesian_product(step)
                    .map(|((min, max), step)| {
                        let options = ((max - min) / step).to_usize();
                        let i = fastrand::usize(0..options);
                        let v = min + step * T::from_usize(i);
                        v
                    });

                (v, r_0 || r_1 || r_2)
            },
        }
    }

    

    pub fn cut_loop(&mut self, to_index: usize) {
        match self {
            NumberSpaceTemplate::NumberRange { min, max, step } => {
                min.cut_loop(to_index);
                max.cut_loop(to_index);
                step.cut_loop(to_index);
            },
        }
    }
}
