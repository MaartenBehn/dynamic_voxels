use octa_force::egui::Ui;

use super::data_type::ComposeDataType;

#[derive(Debug, Clone)]
pub enum ComposeNodeGroupe {
    Const,

    NumberSet,
    
    PositionSet,
}

#[derive(Debug, Clone)]
pub enum ComposeNodeType {
    Number,
    Position2D,
    Position3D,

    NumberRange,

    GridInVolume,
    GridOnPlane,
    Path,
}

#[derive(Debug, Clone)]
pub struct ComposeNode {
    pub t: ComposeNodeType,
    pub group: ComposeNodeGroupe,
    pub inputs: Vec<ComposeNodeInput>,
    pub outputs: Vec<ComposeNodeOutput>,
}

#[derive(Debug, Clone)]
pub struct ComposeNodeInput {
    pub name: String,
    pub data_type: ComposeDataType,
}

#[derive(Debug, Clone)]
pub struct ComposeNodeOutput {
    pub name: String,
    pub data_type: ComposeDataType,
}

impl ComposeNode { 
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


pub fn get_node_templates() -> Vec<ComposeNode> {
    vec![
        ComposeNode { 
            t: ComposeNodeType::Number, 
            group: ComposeNodeGroupe::Const, 
            inputs: vec![], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Number, 
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::Position2D, 
            group: ComposeNodeGroupe::Const, 
            inputs: vec![], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Position2D, 
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::Position3D, 
            group: ComposeNodeGroupe::Const, 
            inputs: vec![], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "out".to_string(), 
                    data_type: ComposeDataType::Position3D, 
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::NumberRange, 
            group: ComposeNodeGroupe::NumberSet, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "min".to_string(), 
                    data_type: ComposeDataType::Number, 
                },
                ComposeNodeInput { 
                    name: "max".to_string(), 
                    data_type: ComposeDataType::Number, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::NumberSet, 
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::GridInVolume, 
            group: ComposeNodeGroupe::PositionSet, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "volume".to_string(), 
                    data_type: ComposeDataType::Volume3D, 
                },
                ComposeNodeInput { 
                    name: "spacing".to_string(), 
                    data_type: ComposeDataType::Number, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSet, 
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::GridOnPlane, 
            group: ComposeNodeGroupe::PositionSet, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "volume".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                },
                ComposeNodeInput { 
                    name: "spacing".to_string(), 
                    data_type: ComposeDataType::Number, 
                },
            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSet, 
                }
            ], 
        },

        ComposeNode { 
            t: ComposeNodeType::Path, 
            group: ComposeNodeGroupe::PositionSet, 
            inputs: vec![
                ComposeNodeInput { 
                    name: "volume".to_string(), 
                    data_type: ComposeDataType::Volume2D, 
                },
                ComposeNodeInput { 
                    name: "start".to_string(), 
                    data_type: ComposeDataType::Position2D, 
                },
                ComposeNodeInput { 
                    name: "end".to_string(), 
                    data_type: ComposeDataType::Position2D, 
                },
                ComposeNodeInput { 
                    name: "side_variance".to_string(), 
                    data_type: ComposeDataType::Number, 
                },
                            ], 
            outputs: vec![
                ComposeNodeOutput { 
                    name: "set".to_string(), 
                    data_type: ComposeDataType::PositionSet, 
                }
            ], 
        },

    ]
}

