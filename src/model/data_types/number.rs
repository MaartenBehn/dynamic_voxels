use std::{env::vars, iter};

use egui_snarl::InPinId;
use itertools::Itertools;
use octa_force::log::debug;
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser, template_changed::MatchValueData}, composer::{ModelComposer, graph::ComposerGraph, make_template::MakeTemplateData, nodes::{ComposeNode, ComposeNodeType}}, data_types::data_type::T, template::{Template, TemplateIndex, value::{TemplateValue, ValueIndex}}}, util::iter_merger::IM4};

use super::{data_type::ComposeDataType, position::{PositionValue, ValueIndexPosition2D, ValueIndexPosition3D}};

pub type ValueIndexNumber = usize;

#[derive(Debug, Clone, Copy)]
pub struct Hook {
    pub template_index: TemplateIndex,
    pub loop_cut: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum NumberValue {
    Const(T),
    Hook(Hook),
    SplitPosition2D((ValueIndexPosition2D, usize)),
    SplitPosition3D((ValueIndexPosition3D, usize)),
    Position3DTo2D(ValueIndexPosition3D),
}

impl ComposerGraph { 
    pub fn make_number(
        &self, 
        original_node: &ComposeNode, 
        in_index: usize, 
        data: &mut MakeTemplateData,
    ) -> ValueIndexNumber {
        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {

            let value = match &original_node.inputs[in_index].data_type {
                ComposeDataType::Number(v) => {
                    if let Some(v) = v {
                        NumberValue::Const(*v)
                    } else {
                        NumberValue::Const(0.0)
                    }
                },
                _ => unreachable!()
            };
            
            data.add_value(TemplateValue::Number(value)) 
        } else {
            let pin = remotes[0];
            if let Some(value_index) = data.get_value_index_from_node_id(pin.node) {
                data.add_depends_of_value(value_index);
                return value_index;
            } 

            let remote_node = self.snarl.get_node(pin.node).expect("Node of remote not found");

            let value = match remote_node.t {
                ComposeNodeType::NumberRange => {
                    todo!();

                    NumberValue::Hook(Hook {
                        template_index: 0,
                        loop_cut: false,
                    })
                },
                ComposeNodeType::SplitPosition2D => {
                    let pos = self.make_position(remote_node, 0, data);

                    assert!(pin.output <= 1);
                    NumberValue::SplitPosition2D((pos, pin.output))
                },
                ComposeNodeType::SplitPosition3D => {
                    let pos = self.make_position(remote_node, 0, data);

                    assert!(pin.output <= 2);
                    NumberValue::SplitPosition3D((pos, pin.output))
                },
                ComposeNodeType::Position3DTo2D => {
                    NumberValue::Position3DTo2D(self.make_position(remote_node, 0, data))
                },
                _ => {
                    unreachable!()
                }
            };

            data.set_value(pin.node, TemplateValue::Number(value))
        }
    }
}


impl Hook {
    pub fn match_value(
        &self, 
        other: &Hook,
        data: MatchValueData
    ) -> bool {
        dbg!(&data.matched_template_indecies);

        if data.matched_template_indecies.len() < other.template_index {
            false
        } else {
            self.template_index == data.matched_template_indecies[other.template_index]
        }
    }
}

impl NumberValue {
    pub fn match_value(
        &self, 
        other: &NumberValue,
        data: MatchValueData
    ) -> bool {

        match self {
            NumberValue::Const(v1) => {
                match other {
                    NumberValue::Const(v2) => v1 == v2,
                    _ => false
                }
            },
            NumberValue::Hook(hook) => {
                match other {
                    NumberValue::Hook(other_hook) => hook.match_value(other_hook, data),
                    _ => false
                }
            },
            NumberValue::SplitPosition2D((p1, i1)) => {
                match other {
                    NumberValue::SplitPosition2D((p2, i2)) => *i1 == *i2 && 
                        data.match_two_positions2d(*p1, *p2),
                    _ => false
                }
            },
            NumberValue::SplitPosition3D((p1, i1)) => {
                match other {
                    NumberValue::SplitPosition3D((p2, i2)) => *i1 == *i2 && 
                        data.match_two_positions3d(*p1, *p2),
                    _ => false
                }
            },
            NumberValue::Position3DTo2D(p1) => {
                match other {
                    NumberValue::Position3DTo2D(p2) => data.match_two_positions3d(*p1, *p2),
                    _ => false
                }
            },
        }
    }

    pub fn get_value(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser,
    ) -> (SmallVec<[T; 1]>, bool) {

        match self {
            NumberValue::Const(v) => (smallvec::smallvec![*v], false),
            NumberValue::Hook(hook) => {
                todo!();
                //let (i, r) = collapser.get_dependend_number(hook.template_index, get_value_data); 
                //(i.collect(), r)
            }
            NumberValue::SplitPosition2D((position_template, i)) => {
                let (v, r) = collapser.template
                    .get_position2d_value(*position_template)
                    .get_value(get_value_data, collapser);
                let v = v.into_iter().map(|v| v[*i]);

                (v.collect(), r)
            },
            NumberValue::SplitPosition3D((position_template, i)) => {
                let (v, r) = collapser.template
                    .get_position3d_value(*position_template)
                    .get_value(get_value_data, collapser);
                let v = v.into_iter().map(|v| v[*i]);

                (v.collect(), r)
            },
            NumberValue::Position3DTo2D(p) => {
                let (v, r) = collapser.template.get_position3d_value(*p)
                    .get_value(get_value_data, collapser);
                let v = v.into_iter().map(|v| {
                    let a: [T; 3] = v.to_array();
                    a[2]
                });
                
                (v.collect(), r)
            },
        }
    }
}



