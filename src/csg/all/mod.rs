use octa_force::glam::{IVec3, UVec3, Vec3A};

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::{VolumeBounds, VolumeBoundsI, VolumeQureyAABB, VolumeQureyAABBI, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue, VolumeQureyPosValueI}};

#[derive(Clone, Copy, Debug)]
pub struct CSGAll<T> {
    v: T,
}

impl<T> CSGAll<T> {
    pub fn new(v: T) -> Self {
        Self { v }
    }
}

impl<T> VolumeBounds for CSGAll<T> {
    fn calculate_bounds(&mut self) {}
    fn get_bounds(&self) -> AABB { AABB::infinte() }
}

impl<T> VolumeBoundsI for CSGAll<T> {
    fn calculate_bounds_i(&mut self) {}
    fn get_bounds_i(&self) -> AABBI { AABBI::infinte() }
}

impl<T> VolumeQureyPosValid for CSGAll<T> {
    fn is_position_valid_vec3(&self, _: Vec3A) -> bool { true }
}

impl VolumeQureyPosValue for CSGAll<u8> {
    fn get_value(&self, pos: Vec3A) -> u8 { self.v }
}

impl VolumeQureyPosValueI for CSGAll<u8> {
    fn get_value_i(&self, _: IVec3) -> u8 { self.v }
    fn get_value_relative_u(&self, _: UVec3) -> u8 { self.v }
}

impl VolumeQureyAABB for CSGAll<u8> {
    fn get_aabb_value(&self, _: AABB) -> VolumeQureyAABBResult {
        VolumeQureyAABBResult::Full(self.v)
    }
}

impl VolumeQureyAABBI for CSGAll<u8> {
    fn get_aabb_value_i(&self, _: AABBI) -> VolumeQureyAABBResult {
         VolumeQureyAABBResult::Full(self.v)
    }
}
