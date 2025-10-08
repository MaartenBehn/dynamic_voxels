use egui_snarl::InPinId;
use itertools::Itertools;
use octa_force::log::debug;
use smallvec::SmallVec;

use crate::util::{number::Nu, vector::Ve};

use super::{build::BS, collapse::{add_nodes::GetValueData, collapser::{CollapseNodeKey, Collapser}}, data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, position::PositionTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

#[derive(Debug, Clone)]
pub struct Hook {
    pub template_index: TemplateIndex,
    pub loop_cut: bool,
}

#[derive(Debug, Clone)]
pub enum NumberTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    Const(T),
    Hook(Hook),
    SplitPosition2D((Box<PositionTemplate<V2, V2, V3, T, 2>>, usize)),
    SplitPosition3D((Box<PositionTemplate<V3, V2, V3, T, 3>>, usize)),
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> { 
    pub fn make_number(
        &self, 
        original_node: &ComposeNode<B::ComposeType>, 
        in_index: usize, 
        building_template_index: usize,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> NumberTemplate<V2, V3, T> {
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
                ComposeNodeType::NumberRange => NumberTemplate::Hook(Hook {
                    template_index: template.get_index_by_out_pin(pin),
                    loop_cut: false,
                }),
                ComposeNodeType::SplitPosition2D => {
                    let pos = self.make_position(remote_node, 0, building_template_index, template);

                    assert!(pin.output >= 0 && pin.output <= 1);
                    NumberTemplate::SplitPosition2D((Box::new(pos), pin.output))
                },
                ComposeNodeType::SplitPosition3D => {
                    let pos = self.make_position(remote_node, 0, building_template_index, template);

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
            NumberTemplate::Hook(hook) => vec![hook.template_index],
            NumberTemplate::SplitPosition2D((position_template, _)) 
                => position_template.get_dependend_template_nodes().collect_vec(),
            NumberTemplate::SplitPosition3D((position_template, _))
                => position_template.get_dependend_template_nodes().collect_vec(),
        }.into_iter()
    }

    pub fn get_value<B: BS<V2, V3, T>>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> T {

        match self {
            NumberTemplate::Const(v) => *v,
            NumberTemplate::Hook(hook) => collapser.get_dependend_number(hook.template_index, get_value_data.depends),
            NumberTemplate::SplitPosition2D((position_template, i)) => {
                position_template.get_value(get_value_data, collapser)[*i]
            },
            NumberTemplate::SplitPosition3D((position_template, i)) => {
                position_template.get_value(get_value_data, collapser)[*i]
            },
        }
    }

    pub fn cut_loop(&mut self, to_index: usize) {
        match self {
            NumberTemplate::Const(_) => {},
            NumberTemplate::Hook(h) => {
                h.loop_cut |= h.template_index == to_index;
            },
            NumberTemplate::SplitPosition2D((s, _)) => {
                s.cut_loop(to_index);
            },
            NumberTemplate::SplitPosition3D((s, _)) => {
                s.cut_loop(to_index);
            },
        }
    }
}

