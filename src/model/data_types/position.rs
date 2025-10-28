use std::{iter, marker::PhantomData, mem::ManuallyDrop};

use egui_snarl::InPinId;
use itertools::{Either, Itertools};
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::{ComposeNode, ComposeNodeType},  ModelComposer}, template::{update::MakeTemplateData, value::{ComposeTemplateValue, ValueIndex}, ComposeTemplate}}, util::{iter_merger::IM3, number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, number::{Hook, NumberTemplate, ValueIndexNumber}, position_pair_set::ValueIndexPositionPairSet, position_set::{PositionSetTemplate, ValueIndexPositionSet}};

pub type ValueIndexPosition = usize;
pub type ValueIndexPosition2D = usize;
pub type ValueIndexPosition3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionTemplate<V: Ve<T, D>, T: Nu, const D: usize> {
    Const(V),
    FromNumbers([ValueIndexNumber; D]),
    PerPosition(Hook),
    PerPair((Hook, bool)),
    PhantomData(PhantomData<T>),
}

union NumberArrayUnion<const DA: usize, const DB: usize> {
    a: [ValueIndexNumber; DA],
    b: [ValueIndexNumber; DB],
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> { 
    pub fn make_position(
        &self, 
        original_node: &ComposeNode<B::ComposeType>, 
        in_index: usize,
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> ValueIndexPosition {

        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {
            match &original_node.inputs[in_index].data_type {
                ComposeDataType::Position2D(v) => {

                    let value = if let Some(v) = v {
                        PositionTemplate::Const(V2::from_ivec2(*v))
                    } else {
                        PositionTemplate::Const(V2::ZERO)
                    };

                    data.add_value(ComposeTemplateValue::Position2D(value)) 
                },
                ComposeDataType::Position3D(v) => {
                    
                    let value = if let Some(v) = v {
                        PositionTemplate::Const(V3::from_ivec3(*v) )
                    } else {
                        PositionTemplate::Const(V3::ZERO)
                    };

                    data.add_value(ComposeTemplateValue::Position3D(value)) 
                },
                _ => unreachable!()
            }
        } else {
            let pin = remotes[0];
            if let Some(value_index) = data.value_per_node_id.get_value(pin.node) {
                return value_index;
            }

            let remote_node = self.snarl.get_node(pin.node).expect("Node of remote not found");

            let value = match remote_node.t {
                ComposeNodeType::Position2D => {
                    let x = self.make_number(remote_node, 0, data);
                    let y = self.make_number(remote_node, 1, data);

                    ComposeTemplateValue::Position2D(PositionTemplate::FromNumbers([x, y]))
                },
                ComposeNodeType::Position3D => {
                    let x = self.make_number(remote_node, 0, data);
                    let y = self.make_number(remote_node, 1, data);
                    let z = self.make_number(remote_node, 2, data);

                    ComposeTemplateValue::Position3D(PositionTemplate::FromNumbers([x, y, z]))
                },
                ComposeNodeType::PerPosition2D => {
                    let template_index = data.template.get_index_by_out_pin(pin);
                    data.depends.push(template_index);
                    data.creates.push(template_index);

                    ComposeTemplateValue::Position2D(PositionTemplate::PerPosition(Hook {
                        template_index,
                        loop_cut: false,
                    }))
                } 
                ComposeNodeType::PerPosition3D => {
                    let template_index = data.template.get_index_by_out_pin(pin);
                    data.depends.push(template_index);
                    data.creates.push(template_index);

                    ComposeTemplateValue::Position3D(PositionTemplate::PerPosition(Hook {
                        template_index,
                        loop_cut: false,
                    }))
                }
                ComposeNodeType::PerPair2D => {
                    let set = self.make_position_pair_set(
                        self.get_input_remote_pin_by_index(remote_node, 0), data);

                    data.creates.push(data.template.get_position_pair_set_value(set).get_ammount_hook(data.template));
                    ComposeTemplateValue::Position2D(PositionTemplate::PerPair((set, pin.output == 0)))
                }
                ComposeNodeType::PerPair3D => {
                    let set = self.make_position_pair_set(
                        self.get_input_remote_pin_by_index(remote_node, 0), data);

                    data.creates.push(data.template.get_position_pair_set_value(set).get_ammount_hook(data.template));
                    ComposeTemplateValue::Position3D(PositionTemplate::PerPair((set, pin.output == 0)))
                } 
                _ => unreachable!(),
            };
                    
            data.set_value(pin.node, value)
        }
    } 
} 



impl<V: Ve<T, D>, T: Nu, const D: usize> PositionTemplate<V, T, D> {
    pub fn get_value<V2: Ve<T, 2>, V3: Ve<T, 3>, B: BS<V2, V3, T>>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> (SmallVec<[V; 1]>, bool) {

        match self {
            PositionTemplate::Const(v) => (smallvec::smallvec![*v], false),
            PositionTemplate::FromNumbers(n) => {
                let mut r_final = false;

                let n = n
                    .into_iter()
                    .map(|n| {
                        let (i, r) =  template
                            .get_number_value(*n)
                            .get_value(get_value_data, collapser, template);

                        r_final |= r;
                        i
                    })
                    .multi_cartesian_product()
                    .map(|n| V::from_iter(&mut n.into_iter()));

                (n.collect(), r_final)
            },
            PositionTemplate::PerPosition(set) => {
                let (p, r) =  template
                    .get_position_set_value(*set)
                    .get_value(get_value_data, collapser, template);

                (p.collect(), r)
            },
            PositionTemplate::PerPair((set, is_a)) => {
                let (set, r) = template
                    .get_position_pair_set_value(*set)
                    .get_value(get_value_data, collapser, template);

                let p = set.map(|(a, b)| if *is_a { a } else { b });
                (p.collect(), r)
            },
            PositionTemplate::PhantomData(_) => unreachable!(),
        }
    }
}


