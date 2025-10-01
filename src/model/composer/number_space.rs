use egui_snarl::OutPinId;
use octa_force::OctaResult;

use crate::{model::generation::traits::ModelGenerationTypes, util::{number::Nu, vector::Ve}};

use super::{build::BS, nodes::ComposeNodeType, number::NumberTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

#[derive(Debug, Clone)]
pub enum NumberSpaceTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    NumberRange {
        min: NumberTemplate<V2, V3, T>,
        max: NumberTemplate<V2, V3, T>,
        step: NumberTemplate<V2, V3, T>,
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_number_space(&self, pin: OutPinId, building_template_index: usize, template: &ComposeTemplate<V2, V3, T, B>) -> NumberSpaceTemplate<V2, V3, T> {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::NumberRange => {
                let min = self.make_number(node, 0, building_template_index,  template);
                let max = self.make_number(node, 1, building_template_index, template);
                let step = self.make_number(node, 2, building_template_index, template);
                NumberSpaceTemplate::NumberRange { min, max, step }
            },
            _ => unreachable!(),
        }
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> NumberSpaceTemplate<V2, V3, T> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            NumberSpaceTemplate::NumberRange { min, max, step } => {
                min.get_dependend_template_nodes()
                    .chain(max.get_dependend_template_nodes())
                    .chain(step.get_dependend_template_nodes())
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
