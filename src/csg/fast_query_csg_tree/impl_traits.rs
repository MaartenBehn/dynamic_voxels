use octa_force::glam::{UVec3, Vec3, Vec3A, Vec4};


use crate::{util::aabb3d::AABB, volume::{VolumeBounds, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}};

use super::tree::FastQueryCSGTree;

impl<T> VolumeBounds for FastQueryCSGTree<T> {
    fn get_bounds(&self) -> AABB {
        self.aabb
    }

    fn calculate_bounds(&mut self) {}
}

impl VolumeQureyPosValid for FastQueryCSGTree<()> {
    fn is_position_valid_vec3(&self, pos: Vec4) -> bool {
        self.is_pos_valid_internal(pos, 0)
    }
}

impl VolumeQureyPosValue for FastQueryCSGTree<u8> {
    fn get_value(&self, pos: Vec3) -> u8 {
        self.get_pos_internal(Vec4::from((pos, 1.0)), 0)
    }

    fn get_value_a(&self, pos: Vec3A) -> u8 {
        self.get_pos_internal(Vec4::from((pos, 1.0)), 0)
    }

    fn get_value_u(&self, pos: UVec3) -> u8 {
        self.get_value(pos.as_vec3())
    }
}

impl VolumeQureyAABB for FastQueryCSGTree<u8> {
    fn get_aabb_value(&self, aabb: AABB) -> VolumeQureyAABBResult {
       self.get_aabb_internal(aabb, 0) 
    }
}

