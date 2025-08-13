use octa_force::{anyhow::bail, glam::{Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{volume::{VolumeQureyPosValid, VolumeQureyPosValid2D}};

use super::{collapse::{CollapseChildKey, CollapseNodeKey, Collapser}, traits::ModelGenerationTypes};

#[derive(Debug, Clone)]
pub struct PositionSet<T: ModelGenerationTypes> {
    pub rule: PositionSetRule<T>,
    pub positions: SlotMap<CollapseChildKey, Vec3A>,
}

#[derive(Debug, Clone)]
pub enum PositionSetRule<T: ModelGenerationTypes> {
    GridInVolume(GridVolumeData<T>),
    GridOnPlane(GridOnPlaneData<T>),
    Path(Path)
}

#[derive(Debug, Clone)]
pub struct GridVolumeData<T: ModelGenerationTypes> {
    pub volume: T::Volume,
    pub spacing: f32,
}

#[derive(Debug, Clone)]
pub struct GridOnPlaneData<T: ModelGenerationTypes> {
    pub volume: T::Volume2D,
    pub spacing: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub spacing: f32,
    pub side_variance: Vec3A,
    pub start: Vec3A,
    pub end: Vec3A,
}

#[derive(Debug, Clone)]
pub struct IterativeGridData {
    pub spacing: f32,
}

impl<T: ModelGenerationTypes> PositionSet<T> {
    pub fn new_grid_in_volume(volume: T::Volume, spacing: f32) -> Self {
        Self { 
            rule: PositionSetRule::GridInVolume(GridVolumeData {
                spacing,
                volume, 
            }), 
            positions: Default::default(),
        }
    }

    pub fn new_grid_on_plane(volume: T::Volume2D, spacing: f32, height: f32) -> Self {
        Self { 
            rule: PositionSetRule::GridOnPlane(GridOnPlaneData {
                volume, 
                spacing,
                height,
            }), 
            positions: Default::default(),
        }
    }
 
    pub fn new_path(spacing: f32, side_variance: Vec3A, start: Vec3A, end: Vec3A) -> Self {
        Self { 
            rule: PositionSetRule::Path(Path {
                spacing,
                side_variance,
                start,
                end,
            }), 
            positions: Default::default(),
        }
    }
 
    pub fn get_num_positions(&self) -> usize {
        self.positions.len()
    }

    pub fn get_pos(&self, pos_key: CollapseChildKey) -> Vec3A {
        self.positions[pos_key]
    }

    pub fn get_volume_mut(&mut self) -> &mut T::Volume {
        match &mut self.rule {
            PositionSetRule::GridInVolume(d) => &mut d.volume,
            _ => panic!("Not a Position Set that uses a volume.")
        }
    }

    pub fn get_volume2d_mut(&mut self) -> &mut T::Volume2D {
        match &mut self.rule {
            PositionSetRule::GridOnPlane(d) => &mut d.volume,
            _ => panic!("Not a Position Set that uses a volume2d.")
        }
    }

    pub fn is_valid_child(&self, pos_key: CollapseChildKey) -> bool {
        self.positions.contains_key(pos_key)
    }

    pub fn set_volume(&mut self, volume: T::Volume) {
        let PositionSetRule::GridInVolume(data) = &mut self.rule 
        else { panic!("Not a Position Set that uses a volume.") };
        data.volume = volume;
    }

    pub fn set_volume2d(&mut self, volume: T::Volume2D) {
        let PositionSetRule::GridOnPlane(data) = &mut self.rule 
        else { panic!("Not a Position Set that uses a volume 2d.") };
        data.volume = volume;
    }
}


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
