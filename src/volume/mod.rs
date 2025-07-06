use std::fmt::Debug;

use octa_force::glam::{vec4, IVec3, UVec3, Vec3, Vec3A, Vec4, Vec4Swizzles};

use crate::util::aabb::AABB;

pub trait VolumeBounds {
    fn calculate_bounds(&mut self);
    fn get_bounds(&self) -> AABB;
    fn get_offset(&self) -> Vec3A {
        Vec3A::from(self.get_bounds().min)
    }
    fn get_size(&self) -> Vec3A {
        Vec3A::from(self.get_bounds().size())
    }
}


pub trait VolumeRandomPos {
    fn get_random_valid_position(&self, search_size: f32) -> Option<Vec3>;
}

pub trait VolumeGradient {
    fn get_gradient_at_position(&self, pos: Vec3) -> Vec3;
}

pub trait VolumeQureyPosValid: VolumeBounds + Clone + Default + Debug {
    fn is_position_valid_vec3(&self, pos: Vec4) -> bool;
     
    fn get_grid_positions(&mut self, step: f32) -> impl IntoIterator<Item = Vec3> {
        let aabb = self.get_bounds();
        aabb.get_sampled_positions(step).into_iter()
            .filter(|p| self.is_position_valid_vec3(vec4(p.x, p.y, p.z, 1.0)))
    }
}

pub trait VolumeQureyPosValue: VolumeBounds {
    fn get_value(&self, pos: Vec3) -> u8;
    fn get_value_a(&self, pos: Vec3A) -> u8;
    fn get_value_u(&self, pos: UVec3) -> u8;
}

#[derive(Copy, Clone, Debug)]
pub enum VolumeQureyAABBResult {
    Full(u8),
    Mixed,
}

pub trait VolumeQureyAABB: VolumeQureyPosValue {
    fn get_aabb_value(&self, aabb: AABB) -> VolumeQureyAABBResult;
    }

impl VolumeQureyAABBResult {
    pub fn get_value(self) -> u8 {
        match self {
            VolumeQureyAABBResult::Full(v) => v,
            VolumeQureyAABBResult::Mixed => unreachable!(),
        }
    }
}
