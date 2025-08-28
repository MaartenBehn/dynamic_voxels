use egui_snarl::OutPinId;
use octa_force::{anyhow::bail, glam::{Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::{collapse::CollapseChildKey}};

use super::{data_type::ComposeDataType, nodes::ComposeNodeType, number_space::NumberSpace, primitive::{Number, Position2D}, volume_2d::Volume2D, volume_3d::Volume3D, ModelComposer};

#[derive(Debug, Clone)]
pub struct PositionSpace {
    pub rule: PositionSpaceRule,
    pub positions: SlotMap<CollapseChildKey, Vec3A>,
}

#[derive(Debug, Clone)]
pub enum PositionSpaceRule {
    GridInVolume(GridVolumeData),
    GridOnPlane(GridOnPlaneData),
    Path(Path)
}

#[derive(Debug, Clone)]
pub struct GridVolumeData {
    pub volume: Volume3D,
    pub spacing: Number,
}

#[derive(Debug, Clone)]
pub struct GridOnPlaneData {
    pub volume: Volume2D,
    pub spacing: Number,
    pub height: Number,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub spacing: Number,
    pub side_variance: Position2D,
    pub start: Position2D,
    pub end: Position2D,
}

#[derive(Debug, Clone)]
pub struct IterativeGridData {
    pub spacing: f32,
}

impl PositionSpace { 
    pub fn new(rule: PositionSpaceRule) -> Self {
        Self { rule, positions: Default::default() }
    }

    pub fn get_num_positions(&self) -> usize {
        self.positions.len()
    }

    pub fn get_pos(&self, pos_key: CollapseChildKey) -> Vec3A {
        self.positions[pos_key]
    }

    pub fn get_positions_iter(&self) -> impl Iterator<Item = Vec3A> {
        self.positions.values().copied()
    }

    pub fn get_volume_mut(&mut self) -> OctaResult<&mut Volume3D> {
        match &mut self.rule {
            PositionSpaceRule::GridInVolume(d) => Ok(&mut d.volume),
            _ => bail!("Not a Position Set that uses a volume.")
        }
    }

    pub fn get_volume2d_mut(&mut self) -> OctaResult<&mut Volume2D> {
        match &mut self.rule {
            PositionSpaceRule::GridOnPlane(d) => Ok(&mut d.volume),
            _ => bail!("Not a Position Set that uses a volume2d.")
        }
    }

    pub fn is_valid_child(&self, pos_key: CollapseChildKey) -> bool {
        self.positions.contains_key(pos_key)
    }

    pub fn set_volume(&mut self, volume: Volume3D) -> OctaResult<()> {
        let PositionSpaceRule::GridInVolume(data) = &mut self.rule 
        else { bail!("Not a Position Set that uses a volume.") };
        data.volume = volume;

        Ok(())
    }

    pub fn set_volume2d(&mut self, volume: Volume2D) -> OctaResult<()> {
        let PositionSpaceRule::GridOnPlane(data) = &mut self.rule
        else { bail!("Not a Position Set that uses a volume 2d.") };
        data.volume = volume;

        Ok(())
    }
}

/*
impl Path {
    pub fn get_positions(&self) -> Vec<Vec3A> {

        let mut points = vec![self.start];
        let mut current = self.start;

        loop {
            let delta = self.end - current;
            let length = delta.length();
            if length < self.spacing {
                points.push(self.end);
                return points;
            }

            let r = Vec3A::new(fastrand::f32(), fastrand::f32(), fastrand::f32()) * 2.0 - 1.0;
            let side = r * self.side_variance * length;
            let dir = (delta + side).normalize();
            current = current + dir * self.spacing;
            points.push(current);
        }
    }
}
*/

impl ModelComposer {
    pub fn make_pos_space(&self, pin: OutPinId) -> PositionSpace {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::GridInVolume => {
                
                let grid = GridVolumeData {
                    volume: self.make_volume_3d(self.get_input_node_by_type(node, ComposeDataType::Volume3D)),
                    spacing: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None))),
                };
                PositionSpace::new(PositionSpaceRule::GridInVolume(grid))
            },
            ComposeNodeType::GridOnPlane => {
                
                let grid = GridOnPlaneData {
                    volume: self.make_volume_2d(self.get_input_node_by_type(node, ComposeDataType::Volume2D)),
                    spacing: self.make_number(node, 1),
                    height: self.make_number(node, 2),
                };
                PositionSpace::new(PositionSpaceRule::GridOnPlane(grid))
            },
            ComposeNodeType::Path => {
                
                let path = Path {
                    start: self.make_position2d(node, 0),
                    end: self.make_position2d(node, 1),
                    spacing: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None))),
                    side_variance: self.make_position2d(node, 3),
                };
                PositionSpace::new(PositionSpaceRule::Path(path))
            },

            _ => unreachable!(),
        }
    }
}

