use egui_snarl::OutPinId;
use octa_force::{log::debug, OctaResult};
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::ComposeNodeType, template::{Ammount, ComposeTemplate, MakeTemplateData, TemplateIndex}, ModelComposer}}, util::{number::Nu, vector::Ve}};

use super::number::NumberTemplate;

#[derive(Debug, Clone)]
pub enum NumberSpaceTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    NumberRange {
        min: NumberTemplate<V2, V3, T>,
        max: NumberTemplate<V2, V3, T>,
        step: NumberTemplate<V2, V3, T>,
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_number_space(
        &self, 
        pin: OutPinId, 
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> NumberSpaceTemplate<V2, V3, T> {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::NumberRange => {
                let min = self.make_number(node, 0, data);
                let max = self.make_number(node, 1, data);
                let step = self.make_number(node, 2, data);
                NumberSpaceTemplate::NumberRange { min, max, step }
            },
            _ => unreachable!(),
        }
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> NumberSpaceTemplate<V2, V3, T> { 
    pub fn get_value<B: BS<V2, V3, T>>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (T, bool) {
        match &self {
            NumberSpaceTemplate::NumberRange { min, max, step } => {
                let (min, r_0) = min.get_value(get_value_data, collapser);
                let (max, r_1) = max.get_value(get_value_data, collapser);
                let (step, r_2) = step.get_value(get_value_data, collapser);

                let options = ((max - min) / step).to_usize();
                let i = fastrand::usize(0..options);
                let v = min + step * T::from_usize(i);

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
