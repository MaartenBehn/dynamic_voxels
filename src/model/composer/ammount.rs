use egui_snarl::{NodeId, OutPinId};


use super::{data_type::ComposeDataType, dependency_tree::DependencyTree, nodes::ComposeNodeType, primitive::Number, template::{ComposeTemplate, TemplateIndex}, ModelComposer};


#[derive(Debug, Clone)]
pub struct Ammount {
    pub template_index: TemplateIndex,
    pub n: Number,
    pub dependecy_tree: DependencyTree,
}

impl ModelComposer {
    pub fn make_ammount(&self, pin: OutPinId, template_index: TemplateIndex, template: &ComposeTemplate) -> (Ammount, TemplateIndex) {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::OneGlobal => (Ammount { 
                template_index, 
                n: Number::Const(1),
                dependecy_tree: DependencyTree::default(),
            }, 0),

            ComposeNodeType::OnePer => (Ammount { 
                template_index, 
                n: Number::Const(1), 
                dependecy_tree: DependencyTree::default(),
            }, template.get_index_by_out_pin(self.get_input_node_by_type(node, ComposeDataType::Identifier))),

            ComposeNodeType::NPer => (Ammount { 
                template_index, 
                n: self.make_number(node, 1, template),
                dependecy_tree: DependencyTree::default(),
            }, template.get_index_by_out_pin(self.get_input_node_by_type(node, ComposeDataType::Identifier))),

            _ => unreachable!(),
        }
    }
}

impl Ammount {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        self.n.get_dependend_template_nodes()
    }
}
