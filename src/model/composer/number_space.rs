use egui_snarl::OutPinId;

use crate::model::generation::traits::ModelGenerationTypes;

use super::{nodes::ComposeNodeType, primitive::Number, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

#[derive(Debug, Clone)]
pub enum NumberSpace {
    NumberRange {
        min: Number,
        max: Number,
    }
}

impl ModelComposer {
    pub fn make_number_space(&self, pin: OutPinId, template: &ComposeTemplate) -> NumberSpace {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::NumberRange => {
                let min = self.make_number(node, 0, template);
                let max = self.make_number(node, 1, template);
                NumberSpace::NumberRange { min, max }
            },
            _ => unreachable!(),
        }
    }
}

impl NumberSpace {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            NumberSpace::NumberRange { min, max } => {
                min.get_dependend_template_nodes().chain(max.get_dependend_template_nodes())
            },
        }
    }
}
