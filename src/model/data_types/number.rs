use std::iter;

use egui_snarl::InPinId;
use itertools::Itertools;
use octa_force::log::debug;
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::{ComposeNode, ComposeNodeType}, template::{ComposeTemplate, MakeTemplateData, TemplateIndex}, ModelComposer}}, util::{iter_merger::IM4, number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, position::PositionTemplate};

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
        data: &mut MakeTemplateData<V2, V3, T, B>,
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
                ComposeNodeType::NumberRange => {
                    let template_index = data.template.get_index_by_out_pin(pin);
                    data.depends.push(template_index);

                    NumberTemplate::Hook(Hook {
                        template_index,
                        loop_cut: false,
                    })
                },
                ComposeNodeType::SplitPosition2D => {
                    let pos = self.make_position(remote_node, 0, data);

                    assert!(pin.output <= 1);
                    NumberTemplate::SplitPosition2D((Box::new(pos), pin.output))
                },
                ComposeNodeType::SplitPosition3D => {
                    let pos = self.make_position(remote_node, 0, data);

                    assert!(pin.output <= 2);
                    NumberTemplate::SplitPosition2D((Box::new(pos), pin.output))
                },
                _ => unreachable!()
            }
        
            
        }
    }
}



impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> NumberTemplate<V2, V3, T> {
    pub fn get_value<B: BS<V2, V3, T>>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (SmallVec<[T; 1]>, bool) {

        match self {
            NumberTemplate::Const(v) => (smallvec::smallvec![*v], false),
            NumberTemplate::Hook(hook) => {
                let (i, r) = collapser.get_dependend_number(hook.template_index, get_value_data); 
                (i.collect(), r)
            }
            NumberTemplate::SplitPosition2D((position_template, i)) => {
                let (v, r) = position_template.get_value(get_value_data, collapser);
                let v = v.into_iter().map(|v| v[*i]);

                (v.collect(), r)
            },
            NumberTemplate::SplitPosition3D((position_template, i)) => {
                let (v, r) = position_template.get_value(get_value_data, collapser);
                let v = v.into_iter().map(|v| v[*i]);

                (v.collect(), r)
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


