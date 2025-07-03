use octa_force::glam::{IVec3, UVec3, Vec3};

use crate::{aabb::AABB, volume::{VolumeGradient, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeRandomPos}};

use super::tree::{VecCSGTree, AABB_PADDING};

impl VolumeRandomPos for VecCSGTree {
    fn get_random_valid_position(&self, search_size: f32) -> Option<Vec3> {
        self.find_valid_pos(search_size)
    }
}

impl VolumeGradient for VecCSGTree {
    fn get_gradient_at_position(&self, pos: Vec3) -> Vec3 {
        self.get_gradient_at_pos(pos)
    }
}

impl VolumeQureyPosValid for VecCSGTree {
    fn is_position_valid_vec3(&self, pos: Vec3) -> bool {
        self.at_pos(pos)
    }

    fn get_aabb(&mut self) -> AABB {
        if self.nodes.is_empty() {
            return AABB::default();
        }
        
        self.nodes[0].aabb
    }
}

impl VolumeQureyAABB for VecCSGTree {
    fn get_aabb(&self, aabb: AABB) -> VolumeQureyAABBResult {
       VolumeQureyAABBResult::Mixed 
    }

    fn get_size(&self) -> UVec3 {
        if self.nodes.is_empty() {
            return UVec3::ZERO;
        }

        (self.nodes[0].aabb.max - self.nodes[0].aabb.min).as_uvec3()
    }
}

