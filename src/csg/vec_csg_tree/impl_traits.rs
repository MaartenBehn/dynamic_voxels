use core::fmt;

use octa_force::glam::{IVec3, UVec3, Vec3, Vec4};


use crate::{util::aabb3d::AABB, volume::{VolumeBounds, VolumeGradient, VolumeQureyPosValid, VolumeRandomPos}};

use super::tree::{VecCSGTree};

impl<T: Clone> VolumeBounds for VecCSGTree<T> {
    fn calculate_bounds(&mut self) {
        self.set_all_aabbs();
    }

    fn get_bounds(&self) -> AABB {
        if self.nodes.is_empty() {
            return AABB::default();
        }
        
        self.nodes[0].aabb
    }
}

impl<T> VolumeRandomPos for VecCSGTree<T> {
    fn get_random_valid_position(&self, search_size: f32) -> Option<Vec3> {
        self.find_valid_pos(search_size)
    }
}

impl<T> VolumeGradient for VecCSGTree<T> {
    fn get_gradient_at_position(&self, pos: Vec3) -> Vec3 {
        self.get_gradient_at_pos(pos)
    }
}

impl<T: fmt::Debug + Clone> VolumeQureyPosValid for VecCSGTree<T> {
    fn is_position_valid_vec3(&self, pos: Vec4) -> bool {
        self.at_pos(pos)
    }
}

