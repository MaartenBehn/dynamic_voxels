use octa_force::glam::{IVec3, Vec3};

use crate::aabb::AABB;

pub trait Volume: Clone + Default {
    fn get_random_valid_position(&self, search_size: f32) -> Option<Vec3>;

    fn is_position_valid_vec3(&self, pos: Vec3) -> bool;

    fn is_position_valid_ivec3(&self, pos: IVec3) -> bool;

    fn get_gradient_at_position(&self, pos: Vec3) -> Vec3;

    fn get_aabb(&mut self) -> AABB;
} 
