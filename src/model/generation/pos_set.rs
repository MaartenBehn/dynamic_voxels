use octa_force::{anyhow::bail, glam::{Mat4, Vec3}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::fast_query_csg_tree::tree::FastQueryCSGTree, volume::{VolumeQureyPosValid, VolumeQureyPosValid2D}};

use super::{collapse::{CollapseChildKey, CollapseNodeKey, Collapser}, traits::ModelGenerationTypes};

#[derive(Debug, Clone)]
pub struct PositionSet<T: ModelGenerationTypes> {
    pub rule: PositionSetRule<T>,
    pub positions: SlotMap<CollapseChildKey, Vec3>,
}

#[derive(Debug, Clone)]
pub enum PositionSetRule<T: ModelGenerationTypes> {
    GridInVolume(GridVolumeData<T::Volume>),
    GridOnPlane(GridOnPlaneData<T::Volume2D>)
}

#[derive(Debug, Clone)]
pub struct GridVolumeData<V: VolumeQureyPosValid> {
    pub spacing: f32,
    pub volume: V,
}

#[derive(Debug, Clone)]
pub struct GridOnPlaneData<P: VolumeQureyPosValid2D> {
    pub spacing: f32,
    pub volume: P,
    pub height: f32,
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
                spacing,
                volume, 
                height,
            }), 
            positions: Default::default(),
        }
    }

    pub fn get_num_positions(&self) -> usize {
        self.positions.len()
    }

    pub fn get_pos(&self, pos_key: CollapseChildKey) -> Vec3 {
        self.positions[pos_key]
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

