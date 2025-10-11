use std::{iter, mem::ManuallyDrop};

use egui_snarl::InPinId;
use itertools::{Either, Itertools};
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::{ComposeNode, ComposeNodeType}, template::{Ammount, AmmountType, ComposeTemplate, MakeTemplateData, TemplateIndex}, ModelComposer}}, util::{number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, number::{Hook, NumberTemplate}, position_set::PositionSetTemplate};


#[derive(Debug, Clone)]
pub enum PositionTemplate<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> {
    Const(V),
    FromNumbers([NumberTemplate<V2, V3, T>; D]),
    PerPosition(PositionSetTemplate<V2, V3, T>),
}

union FromNumbersUnion<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const DA: usize, const DB: usize> {
    a: ManuallyDrop<[NumberTemplate<V2, V3, T>; DA]>,
    b: ManuallyDrop<[NumberTemplate<V2, V3, T>; DB]>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> { 
    pub fn make_position<V: Ve<T, D>, const D: usize>(
        &self, 
        original_node: &ComposeNode<B::ComposeType>, 
        in_index: usize,
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> PositionTemplate<V, V2, V3, T, D> {

        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {
            match &original_node.inputs[in_index].data_type {
                ComposeDataType::Position2D(v) => {
                    assert_eq!(D, 2);

                    if let Some(v) = v {
                        PositionTemplate::Const(V::from_ivec2(*v) )
                    } else {
                        PositionTemplate::Const(V::ZERO)
                    }
                },
                ComposeDataType::Position3D(v) => {
                    assert_eq!(D, 3);
                    
                    if let Some(v) = v {
                        PositionTemplate::Const(V::from_ivec3(*v) )
                    } else {
                        PositionTemplate::Const(V::ZERO)
                    }
                },
                _ => unreachable!()
            }
        } else {
            let pin = remotes[0];
            let remote_node = self.snarl.get_node(pin.node).expect("Node of remote not found");

            match remote_node.t {
                ComposeNodeType::Position2D => {
                    assert_eq!(D, 2);
                    let x = self.make_number(remote_node, 0, data);
                    let y = self.make_number(remote_node, 1, data);

                    // Cast [NumberTemplate; 2] to [NumberTemplate; D] 
                    // Safety: D is 2
                    let numbers = ManuallyDrop::into_inner(unsafe {
                        FromNumbersUnion{ a: ManuallyDrop::new([x, y]) }.b
                    });
                    PositionTemplate::FromNumbers(numbers)
                },
                ComposeNodeType::Position3D => {
                    assert_eq!(D, 3);
                    let x = self.make_number(remote_node, 0, data);
                    let y = self.make_number(remote_node, 1, data);
                    let z = self.make_number(remote_node, 2, data);

                    // Cast [NumberTemplate; 3] to [NumberTemplate; D] 
                    // Safety: D is 3
                    let numbers = ManuallyDrop::into_inner(unsafe {
                        FromNumbersUnion{ a: ManuallyDrop::new([x, y, z]) }.b
                    });
                    PositionTemplate::FromNumbers(numbers)
                },
                ComposeNodeType::PerPosition2D
                | ComposeNodeType::PerPosition3D => {
                    let set = self.make_position_set(pin, data);

                    data.ammounts.push(AmmountType::PerPosition(set.to_owned()));
                    PositionTemplate::PerPosition(set)
                }
                 
                _ => unreachable!(),
            }                                        
        }
    } 
} 



impl<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> PositionTemplate<V, V2, V3, T, D> {
    pub fn get_value<B: BS<V2, V3, T>>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (V, bool) {

        match self {
            PositionTemplate::Const(v) => (*v, false),
            PositionTemplate::FromNumbers(n) => {
                let mut numbers = [T::ZERO; D];
                let mut r_final = false;
                for i in 0..D {
                    let (n, r) = n[i].get_value(get_value_data, collapser);
                    r_final |= r;

                    numbers[i] = n;
                }
                (V::new(numbers), r_final)
            },
            PositionTemplate::PerPosition(set) => set.get_child_value(get_value_data, collapser),
        }
    }

    pub fn cut_loop(&mut self, to_index: usize) {
        match self {
            PositionTemplate::Const(_) => {},
            PositionTemplate::FromNumbers(numbers) => {
                for number in numbers {
                    number.cut_loop(to_index);
                }
            },
            PositionTemplate::PerPosition(set) => set.cut_loop(to_index),
        }
    }
}

