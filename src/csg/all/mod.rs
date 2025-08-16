use octa_force::glam::{IVec3, UVec3, Vec3A};

use crate::{util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}};

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

impl<V, V2: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V2, T, D> for CSGAll<V> {
    fn calculate_bounds(&mut self) {}
    fn get_bounds(&self) -> AABB<V2, T, D> { AABB::infinte() }
}

impl<V, V2: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValid<V2, T, D> for CSGAll<V> {
    fn is_position_valid(&self, pos: V2) -> bool { true }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValue<V, T, D> for CSGAll<u8> {
    fn get_value(&self, pos: V) -> u8 { self.v }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyAABB<V, T, D> for CSGAll<u8> {
    fn get_aabb_value(&self, aabb: AABB<V, T, D>) -> VolumeQureyAABBResult { VolumeQureyAABBResult::Full(self.v) }
}
