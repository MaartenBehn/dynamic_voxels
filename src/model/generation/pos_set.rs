use octa_force::{anyhow::bail, glam::{Mat4, Vec3}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::fast_query_csg_tree::tree::FastQueryCSGTree, volume::{VolumeQureyPosValid, VolumeQureyPosValid2D}};

use super::{builder::{BU, IT}, collapse::{CollapseChildKey, CollapseNodeKey, Collapser}};

#[derive(Debug, Clone)]
pub struct PositionSet<V: VolumeQureyPosValid, P: VolumeQureyPosValid2D>{
    pub rule: PositionSetRule<V, P>,
    pub positions: SlotMap<CollapseChildKey, Vec3>,
}

#[derive(Debug, Clone)]
pub enum PositionSetRule<V: VolumeQureyPosValid, P: VolumeQureyPosValid2D> {
    GridInVolume(GridVolumeData<V>),
    GridOnPlane(GridOnPlaneData<P>)
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

impl<V: VolumeQureyPosValid, P: VolumeQureyPosValid2D> PositionSet<V, P> {
    pub fn new_grid_in_volume(volume: V, spacing: f32) -> Self {
        Self { 
            rule: PositionSetRule::GridInVolume(GridVolumeData {
                spacing,
                volume, 
            }), 
            positions: Default::default(),
        }
    }

    pub fn new_grid_on_plane(volume: P, spacing: f32, height: f32) -> Self {
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

    pub fn set_volume(&mut self, volume: V) -> OctaResult<()> {
        let PositionSetRule::GridInVolume(data) = &mut self.rule 
        else { bail!("Not a Position Set that uses a volume.") };
        data.volume = volume;
        Ok(())
    }

    pub fn set_volume2d(&mut self, volume: P) -> OctaResult<()> {
        let PositionSetRule::GridOnPlane(data) = &mut self.rule 
        else { bail!("Not a Position Set that uses a volume.") };
        data.volume = volume;
        Ok(())
    }
}

