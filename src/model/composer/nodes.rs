use egui_snarl::NodeId;
use octa_force::egui::Ui;

use crate::{model::data_types::data_type::ComposeDataType, util::{number::Nu, vector::Ve}};

use super::build::ComposeTypeTrait;


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComposeNodeGroupe {
    Const,

    NumberSpace,

    PositionSpace2D,
    PositionSpace3D,

    Volume2D,
    Volume3D,

    Math,

    Template,

    Globals, 

    Build
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

    CircleUnion,
    SphereUnion,

    // Math
    SplitPosition2D,
    SplitPosition3D,

    PositionSet2DTo3D,

    // Ammount 
    OneGlobal,
    OnePer,
    NPer,
    ByPositionSet2D,
    ByPositionSet3D,

    // Template
    TemplatePositionSet2D,
    TemplatePositionSet3D,
    TemplateNumberSet,

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

        ComposeNode::new(ComposeNodeType::CircleUnion, ComposeNodeGroupe::Volume2D)
            .input(ComposeDataType::PositionSet2D, "positions")
            .input(ComposeDataType::Number(Some(10)), "size")
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::SphereUnion, ComposeNodeGroupe::Volume3D)
            .input(ComposeDataType::PositionSet3D, "positions")
            .input(ComposeDataType::Number(Some(10)), "size")
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

        ComposeNode::new(ComposeNodeType::PositionSet2DTo3D, ComposeNodeGroupe::Math)
            .input(ComposeDataType::PositionSet2D, "xy")
            .input(ComposeDataType::Number(None), "z")
            .output(ComposeDataType::PositionSet3D, "xyz"),

        // Ammount
        ComposeNode::new(ComposeNodeType::OneGlobal, ComposeNodeGroupe::Template)
            .output(ComposeDataType::Ammount, "a"),

        ComposeNode::new(ComposeNodeType::OnePer, ComposeNodeGroupe::Template)
            .input(ComposeDataType::Identifier, "identifier")
            .input(ComposeDataType::Number(Some(1)), "n")
            .output(ComposeDataType::Ammount, "a"),

        ComposeNode::new(ComposeNodeType::NPer, ComposeNodeGroupe::Template)
            .input(ComposeDataType::Identifier, "identifier")
            .input(ComposeDataType::Number(Some(1)), "n")
            .output(ComposeDataType::Ammount, "a"),

        ComposeNode::new(ComposeNodeType::ByPositionSet2D, ComposeNodeGroupe::Template)
            .input(ComposeDataType::IdentifierPositionSet2D, "identifier")
            .output(ComposeDataType::Ammount, "a")
            .output(ComposeDataType::Position2D(None), "pos"),

        ComposeNode::new(ComposeNodeType::ByPositionSet3D, ComposeNodeGroupe::Template)
            .input(ComposeDataType::IdentifierPositionSet3D, "identifier")
            .output(ComposeDataType::Ammount, "a")
            .output(ComposeDataType::Position3D(None), "pos"),


        // Template
        ComposeNode::new(ComposeNodeType::TemplateNumberSet, ComposeNodeGroupe::Template)
            .input(ComposeDataType::Ammount, "ammount")
            .input(ComposeDataType::NumberSpace, "space")
            .output(ComposeDataType::IdentifierNumberSet, "identifier")
            .output(ComposeDataType::Number(None), "number"),

        ComposeNode::new(ComposeNodeType::TemplatePositionSet2D, ComposeNodeGroupe::Template)
            .input(ComposeDataType::Ammount, "ammount")
            .input(ComposeDataType::PositionSpace2D, "space")
            .output(ComposeDataType::IdentifierPositionSet2D, "identifier")
            .output(ComposeDataType::PositionSet2D, "positions"),

        ComposeNode::new(ComposeNodeType::TemplatePositionSet3D, ComposeNodeGroupe::Template)
            .input(ComposeDataType::Ammount, "ammount")
            .input(ComposeDataType::PositionSpace3D, "space")
            .output(ComposeDataType::IdentifierPositionSet3D, "identifier")
            .output(ComposeDataType::PositionSet3D, "positions"),
    ]
}

