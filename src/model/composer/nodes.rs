use egui_snarl::NodeId;
use octa_force::egui::Ui;

use crate::{model::data_types::data_type::ComposeDataType, util::{number::Nu, vector::Ve}};

use super::build::ComposeTypeTrait;


#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ComposeNodeGroupe {
    Const,

    NumberSpace,

    PositionSpace2D,
    PositionSpace3D,

    Volume2D,
    Volume3D,

    Math,

    Template,

    Split,

    PairSet,

    Globals, 

    Build,

    Engine,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComposeNodeType<CT: ComposeTypeTrait> {
    Position2D,
    Position3D,

    NumberRange,

    // Volume
    Grid2D,
    Grid3D,
    LeafSpread2D,
    LeafSpread3D,
    Path2D,
    Path3D,

    EmpytVolume2D,
    EmpytVolume3D,
    Sphere,
    Circle,
    Box2D,
    Box3D,

    UnionVolume2D,
    UnionVolume3D,
    CutVolume2D,
    CutVolume3D,
 
    // Math
    SplitPosition2D,
    SplitPosition3D,
    Position2DTo3D,
    Position3DTo2D,
    AddPosition2D,
    AddPosition3D,
    SubPosition2D,
    SubPosition3D,

    // Split 
    PerPosition2D,
    PerPosition3D,
        
    PerPair2D,
    PerPair3D,

    CamPosition,

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

impl<CT: ComposeTypeTrait> ComposeNode<CT> { 
    pub fn new(t: ComposeNodeType<CT>, group: ComposeNodeGroupe) -> Self {
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


pub fn get_node_templates<CT: ComposeTypeTrait>() -> Vec<ComposeNode<CT>> {
    vec![
        ComposeNode::new(ComposeNodeType::Position2D, ComposeNodeGroupe::Const)
            .input(ComposeDataType::Number(None), "x")
            .input(ComposeDataType::Number(None), "y")
            .output(ComposeDataType::Position2D(None), "out"),

        ComposeNode::new(ComposeNodeType::Position3D, ComposeNodeGroupe::Const)
            .input(ComposeDataType::Number(None), "x")
            .input(ComposeDataType::Number(None), "y")
            .input(ComposeDataType::Number(None), "z")
            .output(ComposeDataType::Position3D(None), "out"),

        ComposeNode::new(ComposeNodeType::NumberRange, ComposeNodeGroupe::NumberSpace)
            .input(ComposeDataType::Number(None), "min")
            .input(ComposeDataType::Number(None), "max")
            .input(ComposeDataType::Number(None), "step")
            .output(ComposeDataType::NumberSpace, "out"),

        ComposeNode::new(ComposeNodeType::Grid3D, ComposeNodeGroupe::PositionSpace3D)
            .input(ComposeDataType::Volume3D, "volume")
            .input(ComposeDataType::Number(None), "spacing")
            .output(ComposeDataType::PositionSpace3D, "s"),

        ComposeNode::new(ComposeNodeType::Grid2D, ComposeNodeGroupe::PositionSpace2D)
            .input(ComposeDataType::Volume2D, "volume")
            .input(ComposeDataType::Number(None), "spacing")
            .output(ComposeDataType::PositionSpace2D, "s"),

        ComposeNode::new(ComposeNodeType::LeafSpread3D, ComposeNodeGroupe::PositionSpace3D)
            .input(ComposeDataType::Volume3D, "volume")
            .input(ComposeDataType::Number(None), "samples")
            .output(ComposeDataType::PositionSpace3D, "s"),

        ComposeNode::new(ComposeNodeType::LeafSpread2D, ComposeNodeGroupe::PositionSpace2D)
            .input(ComposeDataType::Volume2D, "volume")
            .input(ComposeDataType::Number(None), "samples")
            .output(ComposeDataType::PositionSpace2D, "s"),

        ComposeNode::new(ComposeNodeType::Path3D, ComposeNodeGroupe::PositionSpace3D)
            .input(ComposeDataType::Position3D(None), "start")
            .input(ComposeDataType::Position3D(None), "end")
            .input(ComposeDataType::Number(Some(10)), "spacing")
            .input(ComposeDataType::Position3D(None), "side variance")
            .output(ComposeDataType::PositionSpace3D, "s"),

        ComposeNode::new(ComposeNodeType::Path2D, ComposeNodeGroupe::PositionSpace2D)
            .input(ComposeDataType::Position2D(None), "start")
            .input(ComposeDataType::Position2D(None), "end")
            .input(ComposeDataType::Number(Some(10)), "spacing")
            .input(ComposeDataType::Position2D(None), "side variance")
            .output(ComposeDataType::PositionSpace2D, "s"),


        // New Volume
        ComposeNode::new(ComposeNodeType::EmpytVolume2D, ComposeNodeGroupe::Volume2D)
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::EmpytVolume3D, ComposeNodeGroupe::Volume3D)
            .output(ComposeDataType::Volume3D, "v"),

        ComposeNode::new(ComposeNodeType::Circle, ComposeNodeGroupe::Volume2D)
            .input(ComposeDataType::Position2D(None), "pos")
            .input(ComposeDataType::Number(Some(10)), "size")
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::Sphere, ComposeNodeGroupe::Volume3D)
            .input(ComposeDataType::Position3D(None), "pos")
            .input(ComposeDataType::Number(Some(10)), "size")
            .output(ComposeDataType::Volume3D, "v"),

        ComposeNode::new(ComposeNodeType::Box2D, ComposeNodeGroupe::Volume2D)
            .input(ComposeDataType::Position2D(None), "pos")
            .input(ComposeDataType::Position2D(None), "size")
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::Box3D, ComposeNodeGroupe::Volume3D)
            .input(ComposeDataType::Position3D(None), "pos")
            .input(ComposeDataType::Position3D(None), "size")
            .output(ComposeDataType::Volume3D, "v"),


        // CSG Operations
        ComposeNode::new(ComposeNodeType::UnionVolume2D, ComposeNodeGroupe::Volume2D)
            .input(ComposeDataType::Volume2D, "a")
            .input(ComposeDataType::Volume2D, "b")
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::UnionVolume3D, ComposeNodeGroupe::Volume3D)
            .input(ComposeDataType::Volume3D, "a")
            .input(ComposeDataType::Volume3D, "b")
            .output(ComposeDataType::Volume3D, "v"),

        ComposeNode::new(ComposeNodeType::CutVolume2D, ComposeNodeGroupe::Volume2D)
            .input(ComposeDataType::Volume2D, "base")
            .input(ComposeDataType::Volume2D, "cut")
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::CutVolume3D, ComposeNodeGroupe::Volume3D)
            .input(ComposeDataType::Volume3D, "base")
            .input(ComposeDataType::Volume3D, "cut")
            .output(ComposeDataType::Volume3D, "v"),
 
        // Math
        ComposeNode::new(ComposeNodeType::SplitPosition2D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::Position2D(None), "position")
            .output(ComposeDataType::Number(None), "x")
            .output(ComposeDataType::Number(None), "y"),

        ComposeNode::new(ComposeNodeType::SplitPosition3D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::Position3D(None), "position")
            .output(ComposeDataType::Number(None), "x")
            .output(ComposeDataType::Number(None), "y")
            .output(ComposeDataType::Number(None), "z"),

        ComposeNode::new(ComposeNodeType::Position2DTo3D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::Position2D(None), "xy")
            .input(ComposeDataType::Number(None), "z")
            .output(ComposeDataType::Position3D(None), "xyz"),

        ComposeNode::new(ComposeNodeType::Position3DTo2D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::Position3D(None), "xyz")
            .output(ComposeDataType::Position2D(None), "xy")
            .output(ComposeDataType::Number(None), "z"),

        ComposeNode::new(ComposeNodeType::AddPosition2D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::Position2D(None), "a")
            .input(ComposeDataType::Position2D(None), "b")
            .output(ComposeDataType::Position2D(None), "out"),

        ComposeNode::new(ComposeNodeType::AddPosition3D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::Position3D(None), "a")
            .input(ComposeDataType::Position3D(None), "b")
            .output(ComposeDataType::Position3D(None), "out"),

        ComposeNode::new(ComposeNodeType::SubPosition2D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::Position2D(None), "a")
            .input(ComposeDataType::Position2D(None), "b")
            .output(ComposeDataType::Position2D(None), "out"),

        ComposeNode::new(ComposeNodeType::SubPosition3D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::Position3D(None), "a")
            .input(ComposeDataType::Position3D(None), "b")
            .output(ComposeDataType::Position3D(None), "out"),
        
        // Set
        ComposeNode::new(ComposeNodeType::PerPosition2D, ComposeNodeGroupe::Split)
            .input(ComposeDataType::PositionSpace2D, "space")
            .output(ComposeDataType::Position2D(None), "position")
            .output(ComposeDataType::Creates, ""),

        ComposeNode::new(ComposeNodeType::PerPosition3D, ComposeNodeGroupe::Split)
            .input(ComposeDataType::PositionSpace3D, "space")
            .output(ComposeDataType::Position3D(None), "position")
            .output(ComposeDataType::Creates, ""),

        // Pair Set
        ComposeNode::new(ComposeNodeType::PerPair2D, ComposeNodeGroupe::PairSet)
            .input(ComposeDataType::PositionSpace2D, "space")
            .input(ComposeDataType::Number(None), "distance")
            .output(ComposeDataType::Position2D(None), "a")
            .output(ComposeDataType::Position2D(None), "b")
            .output(ComposeDataType::Creates, ""),

        ComposeNode::new(ComposeNodeType::PerPair3D, ComposeNodeGroupe::PairSet)
            .input(ComposeDataType::PositionSpace3D, "space")
            .input(ComposeDataType::Number(None), "distance")
            .output(ComposeDataType::Position3D(None), "a")
            .output(ComposeDataType::Position3D(None), "b")
            .output(ComposeDataType::Creates, ""),

        ComposeNode::new(ComposeNodeType::CamPosition, ComposeNodeGroupe::Engine)
            .output(ComposeDataType::Position3D(None), "pos"),
    ]
}

