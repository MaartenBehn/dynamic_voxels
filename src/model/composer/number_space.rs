use egui_snarl::OutPinId;

use crate::model::generation::traits::ModelGenerationTypes;

use super::{nodes::ComposeNodeType, primitive::Number, ModelComposer};

#[derive(Debug, Clone)]
pub enum NumberSpace {
    NumberRange {
        min: Number,
        max: Number,
    }
}

impl ModelComposer {
    pub fn make_number_space(&self, pin: OutPinId) -> NumberSpace {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::NumberRange => {
                let min = self.make_number(node, 0);
                let max = self.make_number(node, 1);
                NumberSpace::NumberRange { min, max }
            },
            _ => unreachable!(),
        }
    }
}
