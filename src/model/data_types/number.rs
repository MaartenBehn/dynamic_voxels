use std::{env::vars, iter};

use egui_snarl::InPinId;
use itertools::Itertools;
use octa_force::log::debug;
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, graph::ComposerGraph, nodes::{ComposeNode, ComposeNodeType}, ModelComposer}, template::{update::MakeTemplateData, value::{TemplateValue, ValueIndex}, Template, TemplateIndex}}, util::{iter_merger::IM4, number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, position::{PositionTemplate, ValueIndexPosition2D, ValueIndexPosition3D}};

pub type ValueIndexNumber = usize;

#[derive(Debug, Clone, Copy)]
pub struct Hook {
    pub template_index: TemplateIndex,
    pub loop_cut: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum NumberTemplate<T: Nu> {
    Const(T),
    Hook(Hook),
    SplitPosition2D((ValueIndexPosition2D, usize)),
    SplitPosition3D((ValueIndexPosition3D, usize)),
    Position3DTo2D(ValueIndexPosition3D),
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposerGraph<V2, V3, T, B> { 
    pub fn make_number(
        &self, 
        original_node: &ComposeNode<B::ComposeType>, 
        in_index: usize, 
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> ValueIndexNumber {
        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {

            let value = match &original_node.inputs[in_index].data_type {
                ComposeDataType::Number(v) => {
                    if let Some(v) = v {
                        NumberTemplate::Const(T::from_i32(*v))
                    } else {
                        NumberTemplate::Const(T::ZERO)
                    }
                },
                _ => unreachable!()
            };
            
            data.add_value(TemplateValue::Number(value)) 
        } else {
            let pin = remotes[0];
            /*
            if let Some(value_index) = data.get_value_index_from_node_id(pin.node) {
                return value_index;
            }
            */

            let remote_node = self.snarl.get_node(pin.node).expect("Node of remote not found");

            let value = match remote_node.t {
                ComposeNodeType::NumberRange => {
                    todo!();

                    NumberTemplate::Hook(Hook {
                        template_index: 0,
                        loop_cut: false,
                    })
                },
                ComposeNodeType::SplitPosition2D => {
                    let pos = self.make_position(remote_node, 0, data);

                    assert!(pin.output <= 1);
                    NumberTemplate::SplitPosition2D((pos, pin.output))
                },
                ComposeNodeType::SplitPosition3D => {
                    let pos = self.make_position(remote_node, 0, data);

                    assert!(pin.output <= 2);
                    NumberTemplate::SplitPosition3D((pos, pin.output))
                },
                ComposeNodeType::Position3DTo2D => {
                    NumberTemplate::Position3DTo2D(self.make_position(remote_node, 0, data))
                },
                _ => {
                    unreachable!()
                }
            };

            data.set_value(pin.node, TemplateValue::Number(value))
        }
    }
}



impl<T: Nu> NumberTemplate<T> {
    pub fn get_value<V2: Ve<T, 2>, V3: Ve<T, 3>, B: BS<V2, V3, T>>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &Template<V2, V3, T, B>
    ) -> (SmallVec<[T; 1]>, bool) {

        match self {
            NumberTemplate::Const(v) => (smallvec::smallvec![*v], false),
            NumberTemplate::Hook(hook) => {
                let (i, r) = collapser.get_dependend_number(hook.template_index, get_value_data); 
                (i.collect(), r)
            }
            NumberTemplate::SplitPosition2D((position_template, i)) => {
                let (v, r) = template
                    .get_position2d_value(*position_template)
                    .get_value(get_value_data, collapser, template);
                let v = v.into_iter().map(|v| v[*i]);

                (v.collect(), r)
            },
            NumberTemplate::SplitPosition3D((position_template, i)) => {
                let (v, r) = template
                    .get_position3d_value(*position_template)
                    .get_value(get_value_data, collapser, template);
                let v = v.into_iter().map(|v| v[*i]);

                (v.collect(), r)
            },
            NumberTemplate::Position3DTo2D(p) => {
                let (v, r) = template.get_position3d_value(*p)
                    .get_value(get_value_data, collapser, template);
                let v = v.into_iter().map(|v| {
                    let a: [T; 3] = v.to_array();
                    a[2]
                });
                
                (v.collect(), r)
            },
        }
    }
}



