use std::iter;

use egui_snarl::{NodeId, OutPinId};
use itertools::Itertools;

use crate::util::{number::Nu, vector::Ve};

use super::{build::BS, data_type::ComposeDataType, dependency_tree::DependencyTree, nodes::{ComposeNode, ComposeNodeType}, number::NumberTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};


#[derive(Debug, Clone)]
pub struct Ammount<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    pub template_index: TemplateIndex,
    pub t: AmmountType<V2, V3, T>,
    pub dependecy_tree: DependencyTree,
}

#[derive(Debug, Clone)]
pub enum AmmountType<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    NPer (NumberTemplate<V2, V3, T>),
    ByPosSpace,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_ammount(&self, pin: OutPinId, template_index: TemplateIndex, template: &ComposeTemplate<V2, V3, T, B>) -> (Ammount<V2, V3, T>, TemplateIndex) {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        let ammount = match &node.t {
            ComposeNodeType::OneGlobal => Ammount { 
                template_index, 
                t: AmmountType::NPer(NumberTemplate::Const(T::ONE)),
                dependecy_tree: DependencyTree::default(),
            },

            ComposeNodeType::OnePer => Ammount { 
                template_index, 
                t: AmmountType::NPer(NumberTemplate::Const(T::ONE)), 
                dependecy_tree: DependencyTree::default(),
            },

            ComposeNodeType::NPer => Ammount { 
                template_index, 
                t: AmmountType::NPer(self.make_number(node, 1,  template_index, template)),
                dependecy_tree: DependencyTree::default(),
            },

            ComposeNodeType::ByPositionSet2D => Ammount { 
                template_index, 
                t: AmmountType::ByPosSpace,
                dependecy_tree: DependencyTree::default(),
            },

            ComposeNodeType::ByPositionSet3D => Ammount { 
                template_index, 
                t: AmmountType::ByPosSpace,
                dependecy_tree: DependencyTree::default(),
            },

            _ => unreachable!(),
        };

        let parent_index = self.get_ammount_parent_index(node, template);
        (ammount, parent_index)
    }

    pub fn get_ammount_parent_index(&self, node: &ComposeNode<B::ComposeType>, template: &ComposeTemplate<V2, V3, T, B>) -> TemplateIndex {
        match &node.t {
            ComposeNodeType::OneGlobal => 0,
            ComposeNodeType::OnePer => template.get_index_by_out_pin(self.get_input_remote_pin_by_type(node, ComposeDataType::Identifier)),
            ComposeNodeType::NPer => template.get_index_by_out_pin(self.get_input_remote_pin_by_type(node, ComposeDataType::Identifier)),

            ComposeNodeType::ByPositionSet2D => 
                template.get_index_by_out_pin(self.get_input_remote_pin_by_type(node, ComposeDataType::IdentifierPositionSet2D)),

            ComposeNodeType::ByPositionSet3D => 
                template.get_index_by_out_pin(self.get_input_remote_pin_by_type(node, ComposeDataType::IdentifierPositionSet3D)),
 
            _ => unreachable!(),
        }
    }

    pub fn get_ammount_child_index(&self, node: &ComposeNode<B::ComposeType>, template: &ComposeTemplate<V2, V3, T, B>) -> TemplateIndex {
        match &node.t {
            ComposeNodeType::ByPositionSet2D => 
                template.get_index_by_in_pin(self.get_output_first_remote_pin_by_index(node, 0)),

            ComposeNodeType::ByPositionSet3D => 
                template.get_index_by_in_pin(self.get_output_first_remote_pin_by_index(node, 0)),
 
            _ => unreachable!(),
        }
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> Ammount<V2, V3, T> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match &self.t {
            AmmountType::NPer(n) => n.get_dependend_template_nodes().collect_vec(),
            AmmountType::ByPosSpace => vec![],
        }.into_iter()
    }
}
