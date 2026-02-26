use std::{env::vars, iter};

use egui_snarl::InPinId;
use itertools::Itertools;
use octa_force::log::debug;
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser, template_changed::MatchValueData}, composer::{ModelComposer, graph::ComposerGraph, nodes::{ComposeNode, ComposeNodeType}}, data_types::data_type::T, template::{Template, TemplateIndex, update::MakeTemplateData, value::{TemplateValue, ValueIndex}}}, util::iter_merger::IM4};

use super::{data_type::ComposeDataType, position::{PositionTemplate, ValueIndexPosition2D, ValueIndexPosition3D}};

pub type ValueIndexNumber = usize;

#[derive(Debug, Clone, Copy)]
pub struct Hook {
    pub template_index: TemplateIndex,
    pub loop_cut: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum NumberTemplate {
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
                        NumberTemplate::Const(*v)
                    } else {
                        NumberTemplate::Const(0.0)
                    }
                },
                _ => unreachable!()
            };
            
            data.add_unmapped_value(TemplateValue::Number(value)) 
        } else {
            let pin = remotes[0];
            if let Some(value_index) = data.get_first_value_index_for_node_id(pin.node) {
                data.add_depends_of_value(value_index);
                return value_index;
            } 

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


impl Hook {
    pub fn match_value(
        &self, 
        other: &Hook,
        match_value_data: MatchValueData
    ) -> bool {
        if match_value_data.matched_template_indecies.len() >= self.template_index {
            false
        } else {
            match_value_data.matched_template_indecies[self.template_index] == other.template_index 
        }
    }
}

impl NumberTemplate {
    pub fn match_value(
        &self, 
        other: &NumberTemplate,
        match_value_data: MatchValueData
    ) -> bool {
        match self {
            NumberTemplate::Const(v) => {
                match other {
                    NumberTemplate::Const(v) => v == v,
                    _ => false
                }
            },
            NumberTemplate::Hook(hook) => {
                match other {
                    NumberTemplate::Hook(other_hook) => hook.match_value(other_hook, match_value_data),
                    _ => false
                }
            },
            NumberTemplate::SplitPosition2D((v_index, i)) => {
                match other {
                    NumberTemplate::SplitPosition2D((other_v_index, other_i)) => *i == *other_i && {
                        match_value_data.template.get_position2d_value(*v_index).match_value(
                            match_value_data.template.get_position2d_value(*other_v_index), 
                            match_value_data)
                    },
                    _ => false
                }
            },
            NumberTemplate::SplitPosition3D((v_index, i)) => {
                match other {
                    NumberTemplate::SplitPosition3D((other_v_index, other_i)) => *i == *other_i && {
                        match_value_data.template.get_position3d_value(*v_index).match_value(
                            match_value_data.template.get_position3d_value(*other_v_index), 
                            match_value_data)
                    },
                    _ => false
                }
            },
            NumberTemplate::Position3DTo2D(v_index) => {
                match other {
                    NumberTemplate::Position3DTo2D(other_v_index) => {
                        match_value_data.template.get_position3d_value(*v_index).match_value(
                            match_value_data.template.get_position3d_value(*other_v_index), 
                            match_value_data)

                    },
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
            NumberTemplate::Const(v) => (smallvec::smallvec![*v], false),
            NumberTemplate::Hook(hook) => {
                todo!();
                //let (i, r) = collapser.get_dependend_number(hook.template_index, get_value_data); 
                //(i.collect(), r)
            }
            NumberTemplate::SplitPosition2D((position_template, i)) => {
                let (v, r) = collapser.template
                    .get_position2d_value(*position_template)
                    .get_value(get_value_data, collapser);
                let v = v.into_iter().map(|v| v[*i]);

                (v.collect(), r)
            },
            NumberTemplate::SplitPosition3D((position_template, i)) => {
                let (v, r) = collapser.template
                    .get_position3d_value(*position_template)
                    .get_value(get_value_data, collapser);
                let v = v.into_iter().map(|v| v[*i]);

                (v.collect(), r)
            },
            NumberTemplate::Position3DTo2D(p) => {
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



