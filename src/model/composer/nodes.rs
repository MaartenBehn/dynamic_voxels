use egui_snarl::NodeId;
use octa_force::egui::Ui;

use crate::{model::data_types::data_type::{ComposeDataType, ComposeNodeGroupe, ComposeNodeType}, voxel::palette::palette::{MATERIAL_ID_BASE, MATERIAL_ID_NONE}};



#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNode {
    pub t: ComposeNodeType,
    pub id: NodeId,
    pub group: ComposeNodeGroupe,
    pub inputs: Vec<ComposeNodeInput>,
    pub outputs: Vec<ComposeNodeOutput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNodeInput {
    pub name: String,
    pub data_type: ComposeDataType,

    #[serde(skip)]
    pub valid: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeNodeOutput {
    pub name: String,
    pub data_type: ComposeDataType,
    
    #[serde(skip)]
    pub valid: bool,
}

impl ComposeNode { 
    pub fn new(t: ComposeNodeType, group: ComposeNodeGroupe) -> Self {
        ComposeNode { 
            t, 
            id: NodeId(usize::MAX),
            group,
            inputs: vec![],
            outputs: vec![],
        }
    }

    pub fn input(mut self, t: ComposeDataType, name: &str) -> Self {
        self.inputs.push(ComposeNodeInput {
            name: name.to_string(),
            data_type: t,
            valid: false,
        });
        self
    }

    pub fn output(mut self, t: ComposeDataType, name: &str) -> Self {
        self.outputs.push(ComposeNodeOutput {
            name: name.to_string(),
            data_type: t,
            valid: false,
        });
        self
    }

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



