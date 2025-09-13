use std::mem::ManuallyDrop;

use egui_snarl::InPinId;
use itertools::Itertools;

use crate::util::{number::Nu, vector::Ve};

use super::{build::BS, collapse::collapser::{CollapseNodeKey, Collapser}, data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, number::NumberTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

#[derive(Debug, Clone)]
pub enum PositionTemplate<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> {
    Const(V),
    Hook(TemplateIndex),
    FromNumbers([NumberTemplate<V2, V3, T>; D]),
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
        template: &ComposeTemplate<V2, V3, T, B>,
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
                ComposeNodeType::TemplatePositionSet2D => PositionTemplate::Hook(template.get_index_by_out_pin(pin)),
                ComposeNodeType::TemplatePositionSet3D => PositionTemplate::Hook(template.get_index_by_out_pin(pin)),
                ComposeNodeType::Position2D => {
                    assert_eq!(D, 2);
                    let x = self.make_number(remote_node, 0, template);
                    let y = self.make_number(remote_node, 1, template);

                    // Cast [NumberTemplate; 2] to [NumberTemplate; D] 
                    // Safety: D is 2
                    let numbers = ManuallyDrop::into_inner(unsafe {
                        FromNumbersUnion{ a: ManuallyDrop::new([x, y]) }.b
                    });
                    PositionTemplate::FromNumbers(numbers)
                },
                ComposeNodeType::Position3D => {
                    assert_eq!(D, 3);
                    let x = self.make_number(remote_node, 0, template);
                    let y = self.make_number(remote_node, 1, template);
                    let z = self.make_number(remote_node, 1, template);

                    // Cast [NumberTemplate; 3] to [NumberTemplate; D] 
                    // Safety: D is 3
                    let numbers = ManuallyDrop::into_inner(unsafe {
                        FromNumbersUnion{ a: ManuallyDrop::new([x, y, z]) }.b
                    });
                    PositionTemplate::FromNumbers(numbers)
                },
                _ => unreachable!(),
            }                                        
        }
    } 
} 



impl<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> PositionTemplate<V, V2, V3, T, D> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            PositionTemplate::Const(_) => vec![],
            PositionTemplate::Hook(index) => vec![*index],
            PositionTemplate::FromNumbers(n) => {
                n.iter()
                    .map(|n| n.get_dependend_template_nodes())
                    .flatten()
                    .collect_vec()
            },
        }.into_iter()
    }

    pub fn get_value<B: BS<V2, V3, T>>(
        &self, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> V {

        match self {
            PositionTemplate::Const(v) => *v,
            PositionTemplate::Hook(i) => collapser.get_dependend_position(*i, depends, collapser),
            PositionTemplate::FromNumbers(n) => {
                let mut numbers = [T::ZERO; D];
                for i in 0..D {
                    numbers[i] = n[i].get_value(depends, collapser);
                }
                V::new(numbers)
            },
        }
    }
}

