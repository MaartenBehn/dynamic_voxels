use octa_force::glam::{IVec3, UVec3, Vec3A};

use crate::{util::{aabb::AABB, aabb3d::AABB3, iaabb3d::AABBI, math_config::MC}, volume::{VolumeBounds, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}};

use super::Base;

#[derive(Clone, Copy, Debug)]
pub struct CSGAll<V> {
    v: V
}

impl<V: Base> CSGAll<V> {
    pub fn new() -> Self {
        CSGAll {
            v: V::base(),
        }
    }
}

impl<V, C: MC<D>, const D: usize> VolumeBounds<C, D> for CSGAll<V> {
    fn calculate_bounds(&mut self) {}
    fn get_bounds(&self) -> AABB<C, D> { AABB::infinte() }
}

impl<V, C: MC<D>, const D: usize> VolumeQureyPosValid<C, D> for CSGAll<V> {
    fn is_position_valid(&self, pos: C::Vector) -> bool { true }
}

impl<C: MC<D>, const D: usize> VolumeQureyPosValue<C, D> for CSGAll<u8> {
    fn get_value(&self, pos: C::Vector) -> u8 { self.v }
}

impl<C: MC<D>, const D: usize> VolumeQureyAABB<C, D> for CSGAll<u8> {
    fn get_aabb_value(&self, aabb: AABB<C, D>) -> VolumeQureyAABBResult { VolumeQureyAABBResult::Full(self.v) }
}
