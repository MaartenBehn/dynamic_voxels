use egui_snarl::ui::PinInfo;
use enum_dispatch::enum_dispatch;
use octa_force::{egui::Color32, glam::{IVec2, IVec3, Vec2, Vec3A}};

use crate::model::{composer::nodes::ComposeNode, data_types::{mesh::{MeshCollapserData, MeshTemplate}, none::{NoneCollapserValue}, number::NumberValue, number_set::NumberSet, number_space::NumberSpaceValue, position::PositionValue, position_pair_set::{PositionPairSet, PositionPairSetValue}, position_set::{PositionSet, PositionSetValue}, position_space::PositionSpaceValue, volume::VolumeValue, voxels::{VoxelCollapserData, VoxelValue}}};
use crate::model::collapse::collapser::CollapseValueT;
use crate::model::composer::output_state::OutputState;

pub type T = f32;
pub type V2 = Vec2;
pub type V3 = Vec3A;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ComposeDataType {
    Number(Option<T>), 
    NumberSpace,
    
    Position2D(Option<V2>), 
    Position3D(Option<V3>), 
    
    PositionSpace2D,
    PositionSpace3D,

    Volume2D,
    Volume3D,
    
    Material([u8; 3]),

    Creates,
}

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

    Engine,

    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComposeNodeType {
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
    Disk,
    Circle,
    Box2D,
    Box3D,
    VolumeMaterial2D,
    VolumeMaterial3D,

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

    Voxels,
    Mesh
}


#[derive(Debug, Clone, Copy)]
pub enum TemplateValue {
    None,
    Number(NumberValue),
    NumberSet(NumberSpaceValue),
    Position2D(PositionValue<V2, 2>),
    Position3D(PositionValue<V3, 3>),
    PositionSet2D(PositionSetValue),
    PositionSet3D(PositionSetValue),
    PositionPairSet2D(PositionPairSetValue),
    PositionPairSet3D(PositionPairSetValue),
    PositionSpace2D(PositionSpaceValue),
    PositionSpace3D(PositionSpaceValue),
    Volume2D(VolumeValue),
    Volume3D(VolumeValue),
    Voxels(VoxelValue),
    Mesh(MeshTemplate),
}

impl TemplateValue {
    pub fn to_collapse_value(&self) -> CollapseValue {
        match self {
            TemplateValue::None => CollapseValue::None(Default::default()),
            TemplateValue::NumberSet(_) => CollapseValue::NumberSet(Default::default()),
            TemplateValue::PositionSet2D(_) => CollapseValue::PositionSet2D(Default::default()),
            TemplateValue::PositionSet3D(_) => CollapseValue::PositionSet3D(Default::default()),
            TemplateValue::PositionPairSet2D(_) => CollapseValue::PositionPairSet2D(Default::default()),
            TemplateValue::PositionPairSet3D(_) => CollapseValue::PositionPairSet3D(Default::default()),
            TemplateValue::Voxels(_) => CollapseValue::Voxels(Default::default()), 
            TemplateValue::Mesh(_) => CollapseValue::Mesh(Default::default()), 
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone)]
#[enum_dispatch(CollapseValueT)]
pub enum CollapseValue {
    NumberSet(NumberSet),
    PositionSet2D(PositionSet<V2, T, 2>),
    PositionSet3D(PositionSet<V3, T, 3>),
    PositionPairSet2D(PositionPairSet<V2, T, 2>),
    PositionPairSet3D(PositionPairSet<V3, T, 3>),
    Voxels(VoxelCollapserData),
    Mesh(MeshCollapserData),
    None(NoneCollapserValue),
}


pub fn get_node_templates() -> Vec<ComposeNode> {
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
            .input(ComposeDataType::Number(Some(10.0)), "spacing")
            .input(ComposeDataType::Position3D(None), "side variance")
            .output(ComposeDataType::PositionSpace3D, "s"),

        ComposeNode::new(ComposeNodeType::Path2D, ComposeNodeGroupe::PositionSpace2D)
            .input(ComposeDataType::Position2D(None), "start")
            .input(ComposeDataType::Position2D(None), "end")
            .input(ComposeDataType::Number(Some(10.0)), "spacing")
            .input(ComposeDataType::Position2D(None), "side variance")
            .output(ComposeDataType::PositionSpace2D, "s"),


        // New Volume
        ComposeNode::new(ComposeNodeType::EmpytVolume2D, ComposeNodeGroupe::Volume2D)
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::EmpytVolume3D, ComposeNodeGroupe::Volume3D)
            .output(ComposeDataType::Volume3D, "v"),

        ComposeNode::new(ComposeNodeType::Circle, ComposeNodeGroupe::Volume2D)
            .input(ComposeDataType::Position2D(None), "pos")
            .input(ComposeDataType::Number(Some(10.0)), "size")
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::Sphere, ComposeNodeGroupe::Volume3D)
            .input(ComposeDataType::Position3D(None), "pos")
            .input(ComposeDataType::Number(Some(10.0)), "size")
            .output(ComposeDataType::Volume3D, "v"),

        ComposeNode::new(ComposeNodeType::Disk, ComposeNodeGroupe::Volume3D)
            .input(ComposeDataType::Position3D(None), "pos")
            .input(ComposeDataType::Number(Some(20.0)), "size")
            .input(ComposeDataType::Number(Some(10.0)), "height")
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

        ComposeNode::new(ComposeNodeType::VolumeMaterial2D, ComposeNodeGroupe::Volume2D)
            .input(ComposeDataType::Volume2D, "v")
            .input(ComposeDataType::Material([255, 255, 255]), "")
            .output(ComposeDataType::Volume2D, "v"),

        ComposeNode::new(ComposeNodeType::VolumeMaterial3D, ComposeNodeGroupe::Volume3D)
            .input(ComposeDataType::Volume3D, "v")
            .input(ComposeDataType::Material([255, 255, 255]), "")
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
            .input(ComposeDataType::Creates, "one set per")
            .output(ComposeDataType::Creates, ""),

        ComposeNode::new(ComposeNodeType::PerPosition3D, ComposeNodeGroupe::Split)
            .input(ComposeDataType::PositionSpace3D, "space")
            .output(ComposeDataType::Position3D(None), "position")
            .input(ComposeDataType::Creates, "one set per")
            .output(ComposeDataType::Creates, ""),

        // Pair Set
        ComposeNode::new(ComposeNodeType::PerPair2D, ComposeNodeGroupe::PairSet)
            .input(ComposeDataType::PositionSpace2D, "space")
            .input(ComposeDataType::Number(None), "distance")
            .output(ComposeDataType::Position2D(None), "a")
            .output(ComposeDataType::Position2D(None), "b")
            .input(ComposeDataType::Creates, "one set per")
            .output(ComposeDataType::Creates, ""),

        ComposeNode::new(ComposeNodeType::PerPair3D, ComposeNodeGroupe::PairSet)
            .input(ComposeDataType::PositionSpace3D, "space")
            .input(ComposeDataType::Number(None), "distance")
            .output(ComposeDataType::Position3D(None), "a")
            .output(ComposeDataType::Position3D(None), "b")
            .input(ComposeDataType::Creates, "one set per")
            .output(ComposeDataType::Creates, ""),

        ComposeNode::new(ComposeNodeType::CamPosition, ComposeNodeGroupe::Engine)
            .output(ComposeDataType::Position3D(None), "pos"),

        ComposeNode::new(ComposeNodeType::Voxels, ComposeNodeGroupe::Output)
                .input(ComposeDataType::Volume3D, "volume")
                .input(ComposeDataType::Position3D(None), "pos")
                .input(ComposeDataType::Creates, "one per"),

        ComposeNode::new(ComposeNodeType::Mesh, ComposeNodeGroupe::Output)
                .input(ComposeDataType::Volume3D, "volume")
                .input(ComposeDataType::Position3D(None), "pos")
                .input(ComposeDataType::Creates, "one per"),
    ]
}


impl PartialEq for ComposeDataType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}


