use octa_force::glam::{Mat4, Vec3};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::volume::VolumeQureyPosValid;

use super::{builder::{BU, IT}, collapse::{CollapseChildKey, CollapseNodeKey, Collapser}};

#[derive(Debug, Clone)]
pub struct PositionSet<V: VolumeQureyPosValid> {
    pub volume: V,
    pub rule: PositionSetRule,
    pub positions: SlotMap<CollapseChildKey, Vec3>,
    pub new_positions: Vec<CollapseChildKey>,
}

#[derive(Debug, Clone)]
pub enum PositionSetRule {
    Grid(GridData),
    IterativeGrid(IterativeGridData),
    Possion { distance: f32 },
}

#[derive(Debug, Clone)]
pub struct GridData {
    pub spacing: f32,
}

#[derive(Debug, Clone)]
pub struct IterativeGridData {
    pub spacing: f32,
}

impl<V: VolumeQureyPosValid> PositionSet<V> {
    pub fn new_grid(volume: V, spacing: f32) -> Self {
        Self { 
            volume, 
            rule: PositionSetRule::Grid(GridData {
                spacing
            }), 
            positions: Default::default(),
            new_positions: vec![],
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
}

