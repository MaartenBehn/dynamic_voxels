use egui_snarl::{NodeId, OutPinId};

use crate::util::{number::Nu, vector::Ve};

use super::{data_type::ComposeDataType, dependency_tree::DependencyTree, nodes::ComposeNodeType, primitive::NumberTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};


#[derive(Debug, Clone)]
pub struct Ammount<T: Nu> {
    pub template_index: TemplateIndex,
    pub n: NumberTemplate<T>,
    pub dependecy_tree: DependencyTree,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> ModelComposer<V2, V3, T> {
    pub fn make_ammount(&self, pin: OutPinId, template_index: TemplateIndex, template: &ComposeTemplate<V2, V3, T>) -> (Ammount<T>, TemplateIndex) {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::OneGlobal => (Ammount { 
                template_index, 
                n: NumberTemplate::Const(T::ONE),
                dependecy_tree: DependencyTree::default(),
            }, 0),

            ComposeNodeType::OnePer => (Ammount { 
                template_index, 
                n: NumberTemplate::Const(T::ONE), 
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

impl<T: Nu> Ammount<T> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        self.n.get_dependend_template_nodes()
    }
}
