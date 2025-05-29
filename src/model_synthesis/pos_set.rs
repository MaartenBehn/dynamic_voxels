use octa_force::glam::{Mat4, Vec3};

use crate::{aabb::AABB, volume::Volume};

#[derive(Debug, Clone)]
pub enum PositionSetRule {
    Grid { spacing: f32 },
    Possion { distance: f32 },
}

#[derive(Debug, Clone)]
pub struct PositionSet<V: Volume> {
    volume: V,
    rule: PositionSetRule, 
}

impl<V: Volume> PositionSet<V> {
    pub fn new(volume: V, rule: PositionSetRule) -> Self {
        Self { volume, rule }
    }

    pub fn get_points(&mut self) -> impl IntoIterator<Item = Vec3> {
        match self.rule {
            PositionSetRule::Grid { spacing } => self.volume.get_grid_positions(spacing),
            PositionSetRule::Possion { distance } => todo!(),
        }
    }
}

impl<V: Volume> Default for PositionSet<V> {
    fn default() -> Self {
        Self { volume: Default::default(), rule: PositionSetRule::Grid { spacing: f32::MAX } }
    }
}

