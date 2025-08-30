use octa_force::{anyhow::bail, glam::{IVec3, Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::composer::{position_space::PositionSpace, template::TemplateIndex, volume_2d::Volume2D, volume_3d::Volume3D}, util::math_config::Int3D};

use super::collapser::{CollapseChildKey, CollapseNodeKey, Collapser};


#[derive(Debug, Clone)]
pub struct PositionSet {
    pub rule: PositionSetRule,
    pub positions: SlotMap<CollapseChildKey, Vec3A>,
}

#[derive(Debug, Clone)]
pub enum PositionSetRule {
    GridInVolume(GridVolumeData),
    GridOnPlane(GridOnPlaneData),
    Path(Path)
}

#[derive(Debug, Clone)]
pub struct GridVolumeData {
    pub volume: CSGTree<(), Int3D, 3>,
    pub spacing: i32,
}

#[derive(Debug, Clone)]
pub struct GridOnPlaneData {
    pub volume: Volume2D,
    pub spacing: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub spacing: i32,
    pub side_variance: IVec3,
    pub start: IVec3,
    pub end: IVec3,
}

#[derive(Debug, Clone)]
pub struct IterativeGridData {
    pub spacing: f32,
}

impl PositionSet {
    pub fn from_space(
        space: &PositionSpace, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser
    ) -> Self {
        todo!()
    }
}
