use egui_snarl::{InPinId, NodeId, OutPinId};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::traits::ModelGenerationTypes, util::{math_config::{MC}, number::Nu, vector::Ve}};

use super::{build::BS, collapse::collapser::{CollapseNode, CollapseNodeKey, Collapser}, data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, template::{ComposeTemplate, TemplateIndex, TemplateNode}, ModelComposer};
use crate::util::vector;
use crate::util::math_config;

#[derive(Debug, Clone, Copy)]
pub enum NumberTemplate<N: Nu> {
    Const(N),
    Hook(TemplateIndex),
}

#[derive(Debug, Clone, Copy)]
pub enum PositionTemplate<V: Ve<T, D>, T: Nu, const D: usize> {
    Const(V),
    Hook(TemplateIndex),
    Phantom(T),
}

#[derive(Debug, Clone, Copy)]
pub struct PositionSetTemplate {
    template_index: TemplateIndex,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> { 
    pub fn make_number(&self, original_node: &ComposeNode<B::ComposeType>, in_index: usize, template: &ComposeTemplate<V2, V3, T, B>) -> NumberTemplate<T> {
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
            assert!(matches!(remote_node.outputs[pin.output].data_type, ComposeDataType::Number(..)));
            NumberTemplate::Hook(template.get_index_by_out_pin(pin))
        }
    }

    pub fn make_position<V: Ve<T, D>, const D: usize>(
        &self, 
        original_node: &ComposeNode<B::ComposeType>, 
        in_index: usize, 
        template: &ComposeTemplate<V2, V3, T, B>,
    ) -> PositionTemplate<V, T, D> {

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
            
            let data_type = remote_node.outputs[pin.output].data_type; 
            match D {
                2 => assert!(matches!(data_type, ComposeDataType::Position2D(..))),
                3 => assert!(matches!(data_type, ComposeDataType::Position3D(..))),
                _ => unreachable!()
            }
                        
            PositionTemplate::Hook(template.get_index_by_out_pin(pin))
        }
    }

    pub fn make_position_set(&self, pin: OutPinId, template: &ComposeTemplate<V2, V3, T, B>) -> PositionSetTemplate {
        PositionSetTemplate{ 
            template_index: template.get_index_by_out_pin(pin)
        }
    }

} 


impl<T: Nu> NumberTemplate<T> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            NumberTemplate::Const(_) => None,
            NumberTemplate::Hook(index) => Some(*index),
        }.into_iter()
    }

    pub fn get_value<V2: Ve<T, 2>, V3: Ve<T, 3>, B: BS<V2, V3, T>>(
        &self, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> T {

        match self {
            NumberTemplate::Const(v) => *v,
            NumberTemplate::Hook(i) => collapser.get_dependend_number(*i, depends, collapser),
        }
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> PositionTemplate<V, T, D> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            PositionTemplate::Const(_) => None,
            PositionTemplate::Hook(index) => Some(*index),
            PositionTemplate::Phantom(..) => unreachable!(),
        }.into_iter()
    }

    pub fn get_value<V2: Ve<T, 2>, V3: Ve<T, 3>, B: BS<V2, V3, T>>(
        &self, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> V {

        match self {
            PositionTemplate::Const(v) => *v,
            PositionTemplate::Hook(i) => collapser.get_dependend_position(*i, depends, collapser),
            PositionTemplate::Phantom(..) => unreachable!(),
        }
    }
}

impl PositionSetTemplate {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex>  {
        Some(self.template_index).into_iter()
    }

    pub fn get_value<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> impl Iterator<Item = V> {
        collapser.get_dependend_position_set(self.template_index, depends, collapser)
    }
}
