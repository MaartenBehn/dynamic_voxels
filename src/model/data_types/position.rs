use std::{iter, marker::PhantomData, mem::ManuallyDrop};

use egui_snarl::InPinId;
use itertools::{Either, Itertools};
use smallvec::SmallVec;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser, template_changed::MatchValueData}, composer::{ModelComposer, graph::ComposerGraph, make_template::MakeTemplateData, nodes::{ComposeNode, ComposeNodeType}}, data_types::data_type::{T, V2, V3}, template::{Template, value::{TemplateValue, ValueIndex}}}, util::{iter_merger::IM3, number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, number::{Hook, NumberValue, ValueIndexNumber}, position_pair_set::PositionPairSetValue, position_set::{PositionSetValue, ValueIndexPositionSet}};

pub type ValueIndexPosition = usize;
pub type ValueIndexPosition2D = usize;
pub type ValueIndexPosition3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionValue<V: Ve<T, D>, const D: usize> {
    Const(V),
    Add((ValueIndexPosition, ValueIndexPosition)),
    Sub((ValueIndexPosition, ValueIndexPosition)),

    FromNumbers([ValueIndexNumber; D]),
    Position2DTo3D((ValueIndexPosition2D, ValueIndexNumber)),
    Position3DTo2D(ValueIndexPosition3D),

    PerPosition(Hook),
    PerPair((Hook, bool)),
    Cam,
    PhantomData(PhantomData<T>),
}

union NumberArrayUnion<const DA: usize, const DB: usize> {
    a: [ValueIndexNumber; DA],
    b: [ValueIndexNumber; DB],
}

union PositionUnion<VA: Ve<T, DA>, VB: Ve<T, DB>, T: Nu, const DA: usize, const DB: usize> {
    a: VA,
    b: VB,
    p: PhantomData<T>,
}

impl ComposerGraph { 
    pub fn make_position(
        &self, 
        original_node: &ComposeNode, 
        in_index: usize,
        data: &mut MakeTemplateData,
    ) -> ValueIndexPosition {

        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {
            match &original_node.inputs[in_index].data_type {
                ComposeDataType::Position2D(v) => {

                    let value = if let Some(v) = v {
                        PositionValue::Const(*v)
                    } else {
                        PositionValue::Const(V2::ZERO)
                    };

                    data.add_value(TemplateValue::Position2D(value)) 
                },
                ComposeDataType::Position3D(v) => {
                    
                    let value = if let Some(v) = v {
                        PositionValue::Const(*v)
                    } else {
                        PositionValue::Const(V3::ZERO)
                    };

                    data.add_value(TemplateValue::Position3D(value)) 
                },
                _ => unreachable!()
            }
        } else {
            let pin = remotes[0];
            
            let node = self.snarl.get_node(pin.node).expect("Node of remote not found");

            if let Some(value_index) = data.get_value_index_from_node_id(pin.node) {
                data.add_depends_of_value(value_index);
                
                match &node.t { 
                    ComposeNodeType::PerPair2D => {
                        let value = data.template.get_position2d_value(value_index);
                        match value {
                            PositionValue::PerPair((hook, _)) => {
                                return data.set_value(node.id, TemplateValue::Position2D(
                                    PositionValue::PerPair((*hook, pin.output == 0))));
                            },
                            _ => unreachable!()
                        }
                        
                    }
                    ComposeNodeType::PerPair3D => {
                        let value = data.template.get_position3d_value(value_index);
                        match value {
                            PositionValue::PerPair((hook, _)) => {
                                return data.set_value(node.id, TemplateValue::Position3D(
                                    PositionValue::PerPair((*hook, pin.output == 0))));
                            },
                            _ => unreachable!()
                        } 
                    }
                    _ => {}
                } 

                return value_index;
            }

            let value = match node.t {
                ComposeNodeType::Position2D => {
                    let x = self.make_number(node, 0, data);
                    let y = self.make_number(node, 1, data);

                    TemplateValue::Position2D(PositionValue::FromNumbers([x, y]))
                },
                ComposeNodeType::Position3D => {
                    let x = self.make_number(node, 0, data);
                    let y = self.make_number(node, 1, data);
                    let z = self.make_number(node, 2, data);

                    TemplateValue::Position3D(PositionValue::FromNumbers([x, y, z]))
                },
                ComposeNodeType::Position3DTo2D => {
                    TemplateValue::Position2D(PositionValue::Position3DTo2D(self.make_position(node, 0, data)))
                },
                ComposeNodeType::Position2DTo3D => {
                    TemplateValue::Position3D(PositionValue::Position2DTo3D((
                        self.make_position(node, 0, data),
                        self.make_number(node, 1, data),
                    )))
                },

                ComposeNodeType::AddPosition2D
                | ComposeNodeType::AddPosition3D => {
                    let a = self.make_position(node, 0, data);
                    let b = self.make_position(node, 1, data);

                    TemplateValue::Position3D(PositionValue::Add((a, b)))
                }
                ComposeNodeType::SubPosition2D
                | ComposeNodeType::SubPosition3D => {
                    let a = self.make_position(node, 0, data);
                    let b = self.make_position(node, 1, data);

                    TemplateValue::Position3D(PositionValue::Sub((a, b)))
                }
                ComposeNodeType::PerPosition2D => {
                    let node_data = self.start_template_node(node, data);

                    let space = self.make_pos_space(node, 0, data); 
                    let value = TemplateValue::PositionSet2D(PositionSetValue::All(space));

                    let value_index = data.set_value(node.id, value);

                    let template_index = node_data.finish_template_node(value_index, data);

                    TemplateValue::Position2D(PositionValue::PerPosition(Hook {
                        template_index,
                        loop_cut: false,
                    }))
                } 
                ComposeNodeType::PerPosition3D => {
                    let node_data = self.start_template_node(node, data);

                    let space = self.make_pos_space(node, 0, data); 
                    let value = TemplateValue::PositionSet3D(PositionSetValue::All(space));

                    let value_index = data.set_value(node.id, value);
                    let template_index = node_data.finish_template_node(value_index, data);

                    TemplateValue::Position3D(PositionValue::PerPosition(Hook {
                        template_index,
                        loop_cut: false,
                    }))
                }
                ComposeNodeType::PerPair2D => {
                    let node_data = self.start_template_node(node, data);

                    let space = self.make_pos_space(node, 0, data); 
                    let distance = self.make_number(node, 1, data); 

                    let value = TemplateValue::PositionPairSet2D(PositionPairSetValue::ByDistance((space, distance)));

                    let value_index = data.set_value(node.id, value);
                    let template_index = node_data.finish_template_node(value_index, data);

                    TemplateValue::Position2D(PositionValue::PerPair((Hook {
                        template_index,
                        loop_cut: false,
                    }, pin.output == 0)))
                }
                ComposeNodeType::PerPair3D => {
                    let node_data = self.start_template_node(node, data);
                    
                    let space = self.make_pos_space(node, 0, data); 
                    let distance = self.make_number(node, 1, data); 

                    let value = TemplateValue::PositionPairSet3D(PositionPairSetValue::ByDistance((space, distance)));
                    
                    let value_index = data.set_value(node.id, value);
                    let template_index = node_data.finish_template_node(value_index, data);

                    TemplateValue::Position3D(PositionValue::PerPair((Hook {
                        template_index,
                        loop_cut: false,
                    }, pin.output == 0)))
                }
                ComposeNodeType::CamPosition => {
                    TemplateValue::Position3D(PositionValue::Cam)
                }
                _ => unreachable!(),
            };
                    
            data.set_value(pin.node, value)
        }
    } 
} 



impl<V: Ve<T, D>, const D: usize> PositionValue<V, D> {
    pub fn match_value(
        &self, 
        other: &PositionValue<V, D>,
        match_value_data: MatchValueData
    ) -> bool {
        todo!() 
    }

    pub fn get_value(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser,
    ) -> (SmallVec<[V; 1]>, bool) {

        match self {
            PositionValue::Const(v) => (smallvec::smallvec![*v], false),
            PositionValue::Add((a, b)) => {
                let (a, r_0) = 
                collapser.template.get_position_value::<V, D>(*a).get_value(get_value_data, collapser);
                let (b, r_1) = 
                collapser.template.get_position_value::<V, D>(*b).get_value(get_value_data, collapser);

                let p = a.into_iter()
                    .cartesian_product(b)
                    .map(|(a, b)| a + b);

                (p.collect(), r_0 || r_1)
            }
            PositionValue::Sub((a, b)) => {
                let (a, r_0) = 
                collapser.template.get_position_value::<V, D>(*a).get_value(get_value_data, collapser);
                let (b, r_1) = 
                collapser.template.get_position_value::<V, D>(*b).get_value(get_value_data, collapser);

                let p = a.into_iter()
                    .cartesian_product(b)
                    .map(|(a, b)| a - b);

                (p.collect(), r_0 || r_1)
            }
            PositionValue::FromNumbers(n) => {
                let mut r_final = false;

                let n = n
                    .into_iter()
                    .map(|n| {
                        let (i, r) = collapser.template
                            .get_number_value(*n)
                            .get_value(get_value_data, collapser);

                        r_final |= r;
                        i
                    })
                    .multi_cartesian_product()
                    .map(|n| V::from_iter(&mut n.into_iter()));

                (n.collect(), r_final)
            },
            PositionValue::PerPosition(hook) => {
                let (p, r) = collapser.get_dependend_position(hook.template_index, get_value_data);

                (p.collect(), r)
            },
            PositionValue::PerPair((hook, is_a)) => {
                let (pair, r) = collapser.get_dependend_position_pair(hook.template_index, get_value_data);

                let p = pair.map(|(a, b)| if *is_a { a } else { b });

                (p.collect(), r)
            },
            PositionValue::Cam => {
                let pos = V::from_vec3(get_value_data.external_input.cam_position);
                (smallvec::smallvec![pos], false)
            },
            PositionValue::Position2DTo3D((p, n)) => {

                let (p, r_0) = collapser.template
                            .get_position2d_value(*p)
                            .get_value(get_value_data, collapser);

                let (n, r_1) = collapser.template
                            .get_number_value(*n)
                            .get_value(get_value_data, collapser);

                debug_assert!(D == 3);
                let p = p.into_iter()
                    .cartesian_product(n)
                    .map(|(p, n)| {
                        let a: [T; 2] = p.to_array(); 
                        let a = V3::new(a[0], a[1], n);
                        unsafe { PositionUnion { a }.b }
                    });

                (p.collect(), r_0 || r_1)
            },
            PositionValue::Position3DTo2D(p) => {
                let (p, r) = collapser.template
                    .get_position3d_value(*p)
                    .get_value(get_value_data, collapser);

                debug_assert!(D == 2);
                let p = p.into_iter()
                    .map(|p| {
                        let a: [T; 3] = p.to_array();
                        let a = V2::new(a[0], a[1]);
                        unsafe { PositionUnion { a }.b }
                    });

                (p.collect(), r)
            },
            PositionValue::PhantomData(_) => unreachable!(),
        }
    }
}


