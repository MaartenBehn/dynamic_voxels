use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A, Vec4};


use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::{VolumeBounds, VolumeBoundsI, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue, VolumeQureyPosValueI}};

use super::tree::FastQueryCSGTree;

impl<T> VolumeBounds for FastQueryCSGTree<T> {
    fn get_bounds(&self) -> AABB {
        self.aabb
    }

    fn calculate_bounds(&mut self) {}
}

impl<T> VolumeBoundsI for FastQueryCSGTree<T> {
    fn get_bounds_i(&self) -> AABBI {
        self.aabbi
    }

    fn calculate_bounds(&mut self) {}
}

impl VolumeQureyPosValid for FastQueryCSGTree<()> {
    fn is_position_valid_vec3(&self, pos: Vec4) -> bool {
        self.is_pos_valid_internal(pos, self.root)
    }
}

impl VolumeQureyPosValue for FastQueryCSGTree<u8> {
    fn get_value(&self, pos: Vec3A) -> u8 {
        self.get_pos_internal(Vec4::from((pos, 1.0)), self.root)
    }
}

impl VolumeQureyPosValueI for FastQueryCSGTree<u8> {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        self.get_pos_internal_i(pos, self.root)
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        unreachable!()
    }
}

impl VolumeQureyAABB for FastQueryCSGTree<u8> {
    fn get_aabb_value(&self, aabb: AABB) -> VolumeQureyAABBResult {
       self.get_aabb_internal(aabb, self.root) 
    }
}

