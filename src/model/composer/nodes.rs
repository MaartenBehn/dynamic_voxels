use egui_snarl::NodeId;
use octa_force::egui::Ui;

use crate::util::{number::Nu, vector::Ve};

use super::{build::BS, data_type::ComposeDataType};

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
pub enum ComposeNodeType<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
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

    Build(B::ComposeType)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNode<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub t: ComposeNodeType<V2, V3, T, B>,
    pub id: NodeId,
    pub group: ComposeNodeGroupe,
    pub inputs: Vec<ComposeNodeInput>,
    pub outputs: Vec<ComposeNodeOutput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNodeInput {
    pub name: String,
    pub data_type: ComposeDataType,

}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNodeOutput {
    pub name: String,
    pub data_type: ComposeDataType,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeNode<V2, V3, T, B> { 
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


pub fn get_node_templates<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>>() -> Vec<ComposeNode<V2, V3, T, B>> {
    vec![ 
        ComposeNode { 
            t: ComposeNodeType::Position2D, 
            id: NodeId(usize::MAX),
            group: ComposeNodeGroupe::Const, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "x".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
                ComposeNodeInput { 
                    name: "y".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Position2D(None), 
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
                },
                ComposeNodeInput { 
                    name: "y".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
                ComposeNodeInput { 
                    name: "z".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Position3D(None), 
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
                },
                ComposeNodeInput { 
                    name: "max".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
                ComposeNodeInput { 
                    name: "step".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::NumberSpace, 
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
                },
                ComposeNodeInput { 
                    name: "spacing".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSpace, 
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
                },
                ComposeNodeInput { 
                    name: "spacing".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
                ComposeNodeInput { 
                    name: "height".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSpace, 
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
                },
                ComposeNodeInput { 
                    name: "end".to_string(), 
                    data_type: ComposeDataType::Position2D(None), 
                },
                ComposeNodeInput { 
                    name: "spacing".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
                ComposeNodeInput { 
                    name: "side_variance".to_string(), 
                    data_type: ComposeDataType::Position2D(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSpace, 
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
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
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
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
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
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Position3D(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
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
                },
                ComposeNodeInput { 
                    name: "b".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
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
                },
                ComposeNodeInput { 
                    name: "b".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
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
                },
                ComposeNodeInput { 
                    name: "cut".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
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
                },
                ComposeNodeInput { 
                    name: "cut".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
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
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
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
                },
                ComposeNodeInput { 
                    name: "size".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
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
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
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
                }
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Ammount, 
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
                },
                ComposeNodeInput { 
                    name: "number".to_string(), 
                    data_type: ComposeDataType::Number(None), 
                }
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Ammount, 
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
                },
                ComposeNodeInput { 
                    name: "space".to_string(), 
                    data_type: ComposeDataType::PositionSpace, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "identifier".to_string(), 
                    data_type: ComposeDataType::Identifier, 
                },
                ComposeNodeOutput { 
                    name: "positions".to_string(), 
                    data_type: ComposeDataType::PositionSet, 
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
                },
                ComposeNodeInput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::NumberSpace, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "identifier".to_string(), 
                    data_type: ComposeDataType::Identifier, 
                },
                ComposeNodeOutput { 
                    name: "number".to_string(), 
                    data_type: ComposeDataType::Number(None), 
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
                },
                ComposeNodeInput { 
                    name: "volume".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "identifier".to_string(), 
                    data_type: ComposeDataType::Identifier, 
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
                }
            ], 
        },
    ]
}

