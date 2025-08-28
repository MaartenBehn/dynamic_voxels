use egui_snarl::{InPinId, NodeId, OutPinId};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::traits::ModelGenerationTypes, util::math_config::{Int2D, Int3D}};

use super::{data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, template::TemplateIndex, ModelComposer};

#[derive(Debug, Clone, Copy)]
pub enum Number {
    Const(i32),
    Hook(OutPinId),
}

#[derive(Debug, Clone, Copy)]
pub enum Position2D {
    Const(IVec2),
    Hook(OutPinId),
}

#[derive(Debug, Clone, Copy)]
pub enum Position3D {
    Const(IVec3),
    Hook(OutPinId),
}

#[derive(Debug, Clone, Copy)]
pub enum PositionSet {
    Hook(OutPinId),
}

impl ModelComposer { 
    pub fn make_number(&self, original_node: &ComposeNode, in_index: usize) -> Number {
        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {
            match &original_node.inputs[in_index].data_type {
                ComposeDataType::Number(v) => {
                    if let Some(v) = v {
                        Number::Const(*v)
                    } else {
                        Number::Const(0)
                    }
                },
                _ => unreachable!()
            }
        } else {
            let pin = remotes[0];
            let remote_node = self.snarl.get_node(pin.node).expect("Node of remote not found");
            assert!(matches!(remote_node.outputs[pin.output].data_type, ComposeDataType::Number(..)));
            Number::Hook(pin)
        }
    }

    pub fn make_position2d(&self, original_node: &ComposeNode, in_index: usize) -> Position2D {
        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {
            match &original_node.inputs[in_index].data_type {
                ComposeDataType::Position2D(v) => {
                    if let Some(v) = v {
                        Position2D::Const(*v)
                    } else {
                        Position2D::Const(IVec2::ZERO)
                    }
                },
                _ => unreachable!()
            }
        } else {
            let pin = remotes[0];
            let remote_node = self.snarl.get_node(pin.node).expect("Node of remote not found");
            assert!(matches!(remote_node.outputs[pin.output].data_type, ComposeDataType::Position2D(..)));
            Position2D::Hook(pin)
        }
    }

    pub fn make_position3d(&self, original_node: &ComposeNode, in_index: usize) -> Position3D {
        let remotes = self.snarl.in_pin(InPinId{ node: original_node.id, input: in_index }).remotes;
        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", original_node.t);
        }

        if remotes.is_empty() {
            match &original_node.inputs[in_index].data_type {
                ComposeDataType::Position3D(v) => {
                    if let Some(v) = v {
                        Position3D::Const(*v)
                    } else {
                        Position3D::Const(IVec3::ZERO)
                    }
                },
                _ => unreachable!()
            }
        } else {
            let pin = remotes[0];
            let remote_node = self.snarl.get_node(pin.node).expect("Node of remote not found");
            assert!(matches!(remote_node.outputs[pin.output].data_type, ComposeDataType::Position3D(..)));
            Position3D::Hook(pin)
        }
    }
    pub fn make_position_set(&self, pin: OutPinId) -> PositionSet {
        PositionSet::Hook(pin)
    }

} 

