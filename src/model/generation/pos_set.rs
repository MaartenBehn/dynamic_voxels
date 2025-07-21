use octa_force::glam::{Mat4, Vec3};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::volume::VolumeQureyPosValid;

use super::collapse::{CollapseChildKey, CollapseNodeKey};

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
    spacing: f32,
}

#[derive(Debug, Clone)]
pub struct IterativeGridData {
    spacing: f32,
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

    pub fn collapse(&mut self) {
        match &self.rule {
            PositionSetRule::Grid(grid_data) => {

                let mut new_positions = self.volume.get_grid_positions(grid_data.spacing).collect::<Vec<_>>();
                self.positions.retain(|_, p| {
                    if let Some(i) = new_positions.iter().position(|t| *t == *p) {
                        new_positions.swap_remove(i);
                        false
                    } else {
                        true
                    }
                });
                self.new_positions = new_positions.iter()
                    .map(|p| self.positions.insert(*p))
                    .collect();
            },
            PositionSetRule::Possion { distance } => todo!(),
            PositionSetRule::IterativeGrid(iterative_grid_data) => todo!(),
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

