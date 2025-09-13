use egui_snarl::InPinId;
use itertools::Itertools;
use smallvec::SmallVec;

use crate::util::{number::Nu, vector::Ve};

use super::{build::BS, collapse::collapser::{CollapseNodeKey, Collapser}, data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, position::PositionTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};


#[derive(Debug, Clone)]
pub enum NumberTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    Const(T),
    Hook(TemplateIndex),
    SplitPosition2D((Box<PositionTemplate<V2, V2, V3, T, 2>>, usize)),
    SplitPosition3D((Box<PositionTemplate<V3, V2, V3, T, 3>>, usize)),
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> { 
    pub fn make_number(&self, original_node: &ComposeNode<B::ComposeType>, in_index: usize, template: &ComposeTemplate<V2, V3, T, B>) -> NumberTemplate<V2, V3, T> {
        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {
            match &original_node.inputs[in_index].data_type {
                ComposeDataType::Number(v) => {
                    if let Some(v) = v {
                        NumberTemplate::Const(T::from_i32(*v))
                    } else {
                        NumberTemplate::Const(T::ZERO)
                    }
                },
                _ => unreachable!()
            }
        } else {
            let pin = remotes[0];
            let remote_node = self.snarl.get_node(pin.node).expect("Node of remote not found");

            match remote_node.t {
                ComposeNodeType::NumberRange => NumberTemplate::Hook(template.get_index_by_out_pin(pin)),
                ComposeNodeType::SplitPosition2D => {
                    let pos = self.make_position(remote_node, 0, template);

                    assert!(pin.output >= 0 && pin.output <= 1);
                    NumberTemplate::SplitPosition2D((Box::new(pos), pin.output))
                },
                ComposeNodeType::SplitPosition3D => {
                    let pos = self.make_position(remote_node, 0, template);

                    assert!(pin.output >= 0 && pin.output <= 2);
                    NumberTemplate::SplitPosition2D((Box::new(pos), pin.output))
                },
                _ => unreachable!()
            }
        
            
        }
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> NumberTemplate<V2, V3, T> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            NumberTemplate::Const(_) => vec![],
            NumberTemplate::Hook(index) => vec![*index],
            NumberTemplate::SplitPosition2D((position_template, _)) 
                => position_template.get_dependend_template_nodes().collect_vec(),
            NumberTemplate::SplitPosition3D((position_template, _))
                => position_template.get_dependend_template_nodes().collect_vec(),
        }.into_iter()
    }

    pub fn get_value<B: BS<V2, V3, T>>(
        &self, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> T {

        match self {
            NumberTemplate::Const(v) => *v,
            NumberTemplate::Hook(i) => collapser.get_dependend_number(*i, depends, collapser),
            NumberTemplate::SplitPosition2D((position_template, i)) => {
                position_template.get_value(depends, collapser)[*i]
            },
            NumberTemplate::SplitPosition3D((position_template, i)) => {
                position_template.get_value(depends, collapser)[*i]
            },
        }
    }
}

