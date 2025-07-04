use octa_force::glam::{UVec3, Vec3};


use crate::{util::aabb::AABB, volume::{VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid}};

use super::tree::FastPosQueryCSGTree;

impl VolumeQureyPosValid for FastPosQueryCSGTree {
    fn is_position_valid_vec3(&self, pos: octa_force::glam::Vec3) -> bool {
        self.at_pos_internal(pos, 0)
    }

    fn get_aabb(&mut self) -> AABB {
        self.aabb
    }
}

impl VolumeQureyAABB for FastPosQueryCSGTree {
    fn get_aabb(&self, aabb: AABB) -> VolumeQureyAABBResult {
       VolumeQureyAABBResult::Mixed 
    }

    fn get_offset(&self) -> Vec3 {
        self.aabb.min
    }

    fn get_size(&self) -> Vec3 {
        self.aabb.max - self.aabb.min
    }
}
