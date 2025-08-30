use egui_snarl::OutPinId;
use octa_force::OctaResult;

use crate::{model::generation::traits::ModelGenerationTypes, util::{number::Nu, vector::Ve}};

use super::{build::BS, nodes::ComposeNodeType, primitive::NumberTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

#[derive(Debug, Clone)]
pub enum NumberSpaceTemplate<T: Nu> {
    NumberRange {
        min: NumberTemplate<T>,
        max: NumberTemplate<T>,
        step: NumberTemplate<T>,
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_number_space(&self, pin: OutPinId, template: &ComposeTemplate<V2, V3, T, B>) -> NumberSpaceTemplate<T> {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::NumberRange => {
                let min = self.make_number(node, 0, template);
                let max = self.make_number(node, 1, template);
                let step = self.make_number(node, 2, template);
                NumberSpaceTemplate::NumberRange { min, max, step }
            },
            _ => unreachable!(),
        }
    }
}

impl<T: Nu> NumberSpaceTemplate<T> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            NumberSpaceTemplate::NumberRange { min, max, step } => {
                min.get_dependend_template_nodes()
                    .chain(max.get_dependend_template_nodes())
                    .chain(step.get_dependend_template_nodes())
            },
        }
    }
}
