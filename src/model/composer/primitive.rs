use egui_snarl::{InPinId, NodeId, OutPinId};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::traits::ModelGenerationTypes, util::math_config::{Int2D, Int3D}};

use super::{data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, template::{ComposeTemplate, TemplateIndex, TemplateNode}, ModelComposer};

#[derive(Debug, Clone, Copy)]
pub enum Number {
    Const(i32),
    Hook(TemplateIndex),
}

#[derive(Debug, Clone, Copy)]
pub enum Position2D {
    Const(IVec2),
    Hook(TemplateIndex),
}

#[derive(Debug, Clone, Copy)]
pub enum Position3D {
    Const(IVec3),
    Hook(TemplateIndex),
}

#[derive(Debug, Clone, Copy)]
pub enum PositionSet {
    Hook(TemplateIndex),
}

impl ModelComposer { 
    pub fn make_number(&self, original_node: &ComposeNode, in_index: usize, template: &ComposeTemplate) -> Number {
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
            Number::Hook(template.get_index_by_out_pin(pin))
        }
    }

    pub fn make_position2d(&self, original_node: &ComposeNode, in_index: usize, template: &ComposeTemplate) -> Position2D {
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
            Position2D::Hook(template.get_index_by_out_pin(pin))
        }
    }

    pub fn make_position3d(&self, original_node: &ComposeNode, in_index: usize, template: &ComposeTemplate) -> Position3D {
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
            Position3D::Hook(template.get_index_by_out_pin(pin))
        }
    }
    pub fn make_position_set(&self, pin: OutPinId, template: &ComposeTemplate) -> PositionSet {
        PositionSet::Hook(template.get_index_by_out_pin(pin))
    }

} 


impl Number {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            Number::Const(_) => None,
            Number::Hook(index) => Some(*index),
        }.into_iter()
    }
}

impl Position2D {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            Position2D::Const(_) => None,
            Position2D::Hook(index) => Some(*index),
        }.into_iter()
    }
}

impl Position3D {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            Position3D::Const(_) => None,
            Position3D::Hook(index) => Some(*index),
        }.into_iter()
    }
}

impl PositionSet {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match self {
            PositionSet::Hook(index) => Some(*index),
        }.into_iter()
    }
}
