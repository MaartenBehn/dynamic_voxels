pub mod magica_voxel;
pub mod remove_trait;

use std::fmt::Debug;

use octa_force::glam::{vec4, IVec3, UVec3, Vec2, Vec3, Vec3A, Vec4, Vec4Swizzles};

use crate::util::{aabb::AABB, aabb2d::AABB2, aabb3d::AABB3, iaabb3d::AABBI, math_config::MC};

pub trait VolumeBounds<C: MC<D>, const D: usize> {
    fn calculate_bounds(&mut self);
    fn get_bounds(&self) -> AABB<C, D>;
    fn get_offset(&self) -> C::Vector {
        self.get_bounds().min()
    }
    fn get_size(&self) -> C::Vector {
        self.get_bounds().size()
    }
}

pub trait VolumeRandomPos {
    fn get_random_valid_position(&self, search_size: f32) -> Option<Vec3>;
}

pub trait VolumeGradient {
    fn get_gradient_at_position(&self, pos: Vec3) -> Vec3;
}

pub trait VolumeQureyPosValid<C: MC<D>, const D: usize>: VolumeBounds<C, D> {
    fn is_position_valid(&self, pos: C::Vector) -> bool;
     
    fn get_grid_positions(&self, step: C::Number) -> impl Iterator<Item = C::Vector> {
        let aabb = self.get_bounds();
        aabb.get_sampled_positions(step).into_iter()
            .filter(|p| self.is_position_valid(*p))
    }
}

pub trait VolumeQureyPosValue<C: MC<D>, const D: usize>: VolumeBounds<C, D> {
    fn get_value(&self, pos: C::Vector) -> u8;
}

#[derive(Copy, Clone, Debug)]
pub enum VolumeQureyAABBResult {
    Full(u8),
    Mixed,
}

pub trait VolumeQureyAABB<C: MC<D>, const D: usize>: VolumeQureyPosValue<C, D> {
    fn get_aabb_value(&self, aabb: AABB<C, D>) -> VolumeQureyAABBResult;
}

impl VolumeQureyAABBResult {
    pub fn get_value(self) -> u8 {
        match self {
            VolumeQureyAABBResult::Full(v) => v,
            VolumeQureyAABBResult::Mixed => unreachable!(),
        }
    }
}
