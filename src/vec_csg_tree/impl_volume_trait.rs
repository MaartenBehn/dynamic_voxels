use octa_force::glam::{IVec3, Vec3};

use crate::volume::Volume;

use super::tree::VecCSGTree;


impl Volume for VecCSGTree {
    fn get_random_valid_position(&self, search_size: f32) -> Option<Vec3> {
        self.find_valid_pos(search_size)
    }

    fn is_position_valid_vec3(&self, pos: Vec3) -> bool {
        self.at_pos(pos)
    }

    fn is_position_valid_ivec3(&self, pos: IVec3) -> bool {
        self.at_pos(pos.as_vec3())
    }

    fn get_gradient_at_position(&self, pos: Vec3) -> Vec3 {
        self.get_gradient_at_pos(pos)
    }
}
