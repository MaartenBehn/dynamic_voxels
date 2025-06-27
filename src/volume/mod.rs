use std::fmt::Debug;

use octa_force::glam::{IVec3, UVec3, Vec3};

use crate::aabb::AABB;

pub trait VolumeRandomPos {
    fn get_random_valid_position(&self, search_size: f32) -> Option<Vec3>;
}

pub trait VolumeGradient {
    fn get_gradient_at_position(&self, pos: Vec3) -> Vec3;
}

pub trait VolumeQureyPosValid: Clone + Default + Debug {
    fn is_position_valid_vec3(&self, pos: Vec3) -> bool;
    
    fn get_aabb(&mut self) -> AABB;
    
    fn get_grid_positions(&mut self, step: f32) -> impl IntoIterator<Item = Vec3> {
        let aabb = self.get_aabb();
        aabb.get_sampled_positions(step).into_iter()
            .filter(|p| self.is_position_valid_vec3(p.to_owned()))
    }
}

pub trait VolumeQureyPosValue {
    fn get_value(&self, pos: UVec3) -> u8;
    fn get_size(&self) -> UVec3;
}

