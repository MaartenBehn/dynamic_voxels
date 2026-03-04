use std::fmt::Debug;

use nalgebra::ComplexField;
use octa_force::glam::{IVec3, Vec3Swizzles};

use crate::util::vector::Ve;


pub trait LODHeuristicT: Sync + Send + Debug + Clone {
    fn lod_level(&self, pos: IVec3) -> u8;
    fn set_center(&mut self, center: IVec3);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LODHeuristicNone {}

impl LODHeuristicT for LODHeuristicNone {
    fn lod_level(&self, pos: IVec3) -> u8 { 1 }

    fn set_center(&mut self, center: IVec3) {}
}

#[derive(Debug, Clone, Copy)]
pub struct LinearLODHeuristicSphere {
    pub center: IVec3,
    pub level_size: i32,
}

impl LODHeuristicT for LinearLODHeuristicSphere {
    fn lod_level(&self, pos: IVec3) -> u8 {
        let delta = pos - self.center.yxz();

        let level = delta.abs().max_element() / self.level_size;
        return level.clamp(1, 255) as u8;
    }

    fn set_center(&mut self, center: IVec3) {
        self.center = center;
    }
}

impl Default for LinearLODHeuristicSphere {
    fn default() -> Self {
        Self { center: Default::default(), level_size: 200 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PowHeuristicSphere {
    pub center: IVec3,
    pub render_dist: f32,
}

impl LODHeuristicT for PowHeuristicSphere {
    fn lod_level(&self, pos: IVec3) -> u8 {
        let delta = pos - self.center.yxz();

        //dbg!(delta.length());
        //let level = delta.abs().max_element() / self.level_size;
        let level = ((delta.length() as f32).powf(0.5) / self.render_dist) as u8;
        //dbg!(level);

        return level.max(1);
    }

    fn set_center(&mut self, center: IVec3) {
        self.center = center;
    }
}

impl Default for PowHeuristicSphere {
    fn default() -> Self {
        Self { center: Default::default(), render_dist: 100.0 }
    }
}


