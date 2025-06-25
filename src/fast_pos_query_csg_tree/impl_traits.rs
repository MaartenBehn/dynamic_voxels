use crate::volume::VolumeQureyPos;

use super::tree::FastPosQueryCSGTree;

impl VolumeQureyPos for FastPosQueryCSGTree {
    fn is_position_valid_vec3(&self, pos: octa_force::glam::Vec3) -> bool {
        self.at_pos_internal(pos, 0)
    }

    fn get_aabb(&mut self) -> crate::aabb::AABB {
        self.aabb
    }
}
