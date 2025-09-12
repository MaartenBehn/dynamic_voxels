use egui_snarl::NodeId;
use octa_force::egui::Ui;

use crate::util::{number::Nu, vector::Ve};

use super::{build::{ComposeTypeTrait, BS}, data_type::ComposeDataType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComposeNodeGroupe {
    Const,

    NumberSpace,

    PositionSpace,

    Volume2D,
    Volume3D,

    Template,

    Globals, 

    Build
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComposeNodeType<CT: ComposeTypeTrait> {
    Position2D,
    Position3D,

    NumberRange,

    GridInVolume,
    GridOnPlane,
    Path,

    EmpytVolume2D,
    EmpytVolume3D,
    Sphere,
    Circle,
    Box,
    VoxelObject,

    UnionVolume2D,
    UnionVolume3D,
    CutVolume2D,
    CutVolume3D,

    CircleUnion,
    SphereUnion,
    VoxelObjectUnion,

    // Ammount 
    OnePer,
    OneGlobal,
    NPer,

    // Template
    TemplatePositionSet,
    TemplateNumberSet,
    BuildObject,

    // Globals 
    PlayerPosition,

    Build(CT)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNode<CT: ComposeTypeTrait> {
    pub t: ComposeNodeType<CT>,
    pub id: NodeId,
    pub group: ComposeNodeGroupe,
    pub inputs: Vec<ComposeNodeInput>,
    pub outputs: Vec<ComposeNodeOutput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNodeInput {
    pub name: String,
    pub data_type: ComposeDataType,
    pub valid: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNodeOutput {
    pub name: String,
    pub data_type: ComposeDataType,
    pub valid: bool,
}

impl<CT: ComposeTypeTrait> ComposeNode<CT> { 
    pub fn title(&self) -> String {
        format!("{:?}", self.t)
    }
}

impl ComposeNodeInput {
    pub fn get_name(&self) -> String {
        format!("{}", self.name)
    }
}

impl ComposeNodeOutput {
    pub fn get_name(&self) -> String {
        format!("{}", self.name)
    }
}


pub fn get_node_templates<CT: ComposeTypeTrait>() -> Vec<ComposeNode<CT>> {
    vec![ 
        ComposeNode { 
            t: ComposeNodeType::Position2D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Const, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "x".to_string(), 
                    data_type: ComposeDataType::Number(None),
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "y".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Position2D(None), 
                    valid: false,
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::Position3D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Const, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "x".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "y".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "z".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Position3D(None), 
                    valid: false,
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::NumberRange, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::NumberSpace, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "min".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "max".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "step".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::NumberSpace, 
                    valid: false,
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::GridInVolume, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::PositionSpace, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "volume".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "spacing".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSpace, 
                    valid: false,
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::GridOnPlane, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::PositionSpace, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "volume".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "spacing".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "height".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSpace, 
                    valid: false,
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::Path, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::PositionSpace, 
            inputs: vec![
                //ComposeNodeInput { 
                //    name: "volume".to_string(), 
                //    data_type: ComposeDataType::Volume2D, 
                //},
                ComposeNodeInput { 
                    name: "start".to_string(), 
                    data_type: ComposeDataType::Position2D(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "end".to_string(), 
                    data_type: ComposeDataType::Position2D(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "spacing".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "side_variance".to_string(), 
                    data_type: ComposeDataType::Position2D(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSpace, 
                    valid: false,
                }
            ], 
        },

        // New Volume
        ComposeNode { 
            t: ComposeNodeType::EmpytVolume2D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume2D, 
            inputs: vec![], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::EmpytVolume3D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume3D, 
            inputs: vec![], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::Sphere, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume3D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "pos".to_string(), 
                    data_type: ComposeDataType::Position3D(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::Circle, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume2D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "pos".to_string(), 
                    data_type: ComposeDataType::Position2D(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::Box, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume3D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "pos".to_string(), 
                    data_type: ComposeDataType::Position3D(None), 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Position3D(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::VoxelObject, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume3D, 
            inputs: vec![], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                }
            ], 
        },


        ComposeNode { 
            t: ComposeNodeType::UnionVolume2D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume2D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "a".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "b".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::UnionVolume3D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume3D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "a".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "b".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::CutVolume2D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume2D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "base".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "cut".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::CutVolume3D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume3D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "base".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "cut".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::SphereUnion, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume3D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "positions".to_string(), 
                    data_type: ComposeDataType::PositionSet, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::CircleUnion, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume2D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "positions".to_string(), 
                    data_type: ComposeDataType::PositionSet, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::VoxelObjectUnion, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Volume3D, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "positions".to_string(), 
                    data_type: ComposeDataType::PositionSet, 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                }
            ], 
        }, 


        // Ammount
        ComposeNode { 
            t: ComposeNodeType::OnePer, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Template, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "identifier".to_string(), 
                    data_type: ComposeDataType::Identifier, 
                    valid: false,
                }
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Ammount, 
                    valid: false,
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::OneGlobal, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Template, 
            inputs: vec![], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Ammount, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::NPer, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Template, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "identifier".to_string(), 
                    data_type: ComposeDataType::Identifier, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "number".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                }
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Ammount, 
                    valid: false,
                }
            ], 
        }, 

        // Template
        ComposeNode { 
            t: ComposeNodeType::TemplatePositionSet, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Template, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "ammount".to_string(), 
                    data_type: ComposeDataType::Ammount, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "space".to_string(), 
                    data_type: ComposeDataType::PositionSpace, 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "identifier".to_string(), 
                    data_type: ComposeDataType::Identifier, 
                    valid: false,
                },
                ComposeNodeOutput { 
                    name: "positions".to_string(), 
                    data_type: ComposeDataType::PositionSet, 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::TemplateNumberSet, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Template, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "ammount".to_string(), 
                    data_type: ComposeDataType::Ammount, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::NumberSpace, 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "identifier".to_string(), 
                    data_type: ComposeDataType::Identifier, 
                    valid: false,
                },
                ComposeNodeOutput { 
                    name: "number".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                    valid: false,
                }
            ], 
        },
        ComposeNode { 
            t: ComposeNodeType::BuildObject, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Template, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "ammount".to_string(), 
                    data_type: ComposeDataType::Ammount, 
                    valid: false,
                },
                ComposeNodeInput { 
                    name: "volume".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                    valid: false,
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "identifier".to_string(), 
                    data_type: ComposeDataType::Identifier, 
                    valid: false,
                },
            ], 
        },


        // Globals
        ComposeNode { 
            t: ComposeNodeType::PlayerPosition, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Globals, 
            inputs: vec![], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Position3D(None), 
                    valid: false,
                }
            ], 
        },
    ]
}

