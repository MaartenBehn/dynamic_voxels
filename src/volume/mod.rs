pub mod magica_voxel;
pub mod remove_trait;

use std::fmt::Debug;

use octa_force::glam::{vec4, IVec3, UVec3, Vec2, Vec3, Vec3A, Vec4, Vec4Swizzles};

use crate::util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve};

pub trait VolumeBounds<V: Ve<T, D>, T: Nu, const D: usize> {
    fn calculate_bounds(&mut self);
    fn get_bounds(&self) -> AABB<V, T, D>;
    fn get_offset(&self) -> V {
        self.get_bounds().min()
    }
    fn get_size(&self) -> V {
        self.get_bounds().size()
    }
}

pub trait VolumeRandomPos {
    fn get_random_valid_position(&self, search_size: f32) -> Option<Vec3>;
}

pub trait VolumeGradient {
    fn get_gradient_at_position(&self, pos: Vec3) -> Vec3;
}

pub trait VolumeQureyPosValid<V: Ve<T, D>, T: Nu, const D: usize>: VolumeBounds<V, T, D> {
    fn is_position_valid(&self, pos: V) -> bool;
     
    fn get_grid_positions(&self, step: T) -> impl Iterator<Item = V> {
        let aabb = self.get_bounds();
        aabb.get_sampled_positions(step).into_iter()
            .filter(|p| self.is_position_valid(*p))
    }
}

pub trait VolumeQureyPosValue<V: Ve<T, D>, T: Nu, const D: usize>: VolumeBounds<V, T, D> {
    fn get_value(&self, pos: V) -> u8;
}

#[derive(Copy, Clone, Debug)]
pub enum VolumeQureyAABBResult {
    Full(u8),
    Mixed,
}

pub trait VolumeQureyAABB<V: Ve<T, D>, T: Nu, const D: usize>: VolumeQureyPosValue<V, T, D> {
    fn get_aabb_value(&self, aabb: AABB<V, T, D>) -> VolumeQureyAABBResult;
}

impl VolumeQureyAABBResult {
    pub fn get_value(self) -> u8 {
        match self {
            VolumeQureyAABBResult::Full(v) => v,
            VolumeQureyAABBResult::Mixed => unreachable!(),
        }
    }
}
