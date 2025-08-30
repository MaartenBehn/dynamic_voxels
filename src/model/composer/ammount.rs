use std::iter;

use egui_snarl::{NodeId, OutPinId};
use itertools::Itertools;

use crate::util::{number::Nu, vector::Ve};

use super::{build::BS, data_type::ComposeDataType, dependency_tree::DependencyTree, nodes::ComposeNodeType, primitive::NumberTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};


#[derive(Debug, Clone)]
pub struct Ammount<T: Nu> {
    pub template_index: TemplateIndex,
    pub t: AmmountType<T>,
    pub dependecy_tree: DependencyTree,
}

#[derive(Debug, Clone)]
pub enum AmmountType<T: Nu> {
    NPer (NumberTemplate<T>),
    ByPosSpace,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_ammount(&self, pin: OutPinId, template_index: TemplateIndex, template: &ComposeTemplate<V2, V3, T, B>) -> (Ammount<T>, TemplateIndex) {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::OneGlobal => (Ammount { 
                template_index, 
                t: AmmountType::NPer(NumberTemplate::Const(T::ONE)),
                dependecy_tree: DependencyTree::default(),
            }, 0),

            ComposeNodeType::OnePer => (Ammount { 
                template_index, 
                t: AmmountType::NPer(NumberTemplate::Const(T::ONE)), 
                dependecy_tree: DependencyTree::default(),
            }, template.get_index_by_out_pin(self.get_input_pin_by_type(node, ComposeDataType::Identifier))),

            ComposeNodeType::NPer => (Ammount { 
                template_index, 
                t: AmmountType::NPer(self.make_number(node, 1, template)),
                dependecy_tree: DependencyTree::default(),
            }, template.get_index_by_out_pin(self.get_input_pin_by_type(node, ComposeDataType::Identifier))),

            _ => unreachable!(),
        }
    }
}

impl<T: Nu> Ammount<T> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self.t {
            AmmountType::NPer(n) => n.get_dependend_template_nodes().collect_vec(),
            AmmountType::ByPosSpace => vec![],
        }.into_iter()
    }
}
