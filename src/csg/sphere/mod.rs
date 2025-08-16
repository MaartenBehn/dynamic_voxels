use octa_force::glam::{vec3, IVec3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::{aabb::{AABB}, math_config::MC, matrix::Ma, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{Base};

#[derive(Clone, Copy, Debug, Default)]
pub struct CSGSphere<V, C: MC<D>, const D: usize> {
    mat: C::Matrix,
    v: V
}

impl<V: Base, C: MC<D>, const D: usize> CSGSphere<V, C, D> {
    pub fn new_sphere(center: C::VectorF, radius: f32) -> Self {
        CSGSphere {
            mat: C::Matrix::from_scale_translation(
                C::VectorF::ONE * radius,
                center,
            ).inverse(),
            v: V::base(),
        }
    }
}

impl<V: Base, C: MC<3>> CSGSphere<V, C, 3> {
    pub fn new_disk(center: C::VectorF, radius: f32, height: f32) -> Self {
        CSGSphere {
            mat: C::Matrix::from_scale_translation(
                C::VectorF::new([radius, radius, height]),
                center,
            ).inverse(),
            v: V::base(),
        }
    }
}

impl<V, C: MC<D>, const D: usize> VolumeBounds<C::Vector, C::Number, D> for CSGSphere<V, C, D> {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<C::Vector, C::Number, D> {
        let mat = self.mat.inverse();
        AABB::from_sphere(&mat)
    }
}

impl<V, C: MC<D>, const D: usize> VolumeQureyPosValid<C::Vector, C::Number, D> for CSGSphere<V, C, D> {
    fn is_position_valid(&self, pos: C::Vector) -> bool {
        let pos = self.mat.mul_vector(C::to_vector_f(pos));
        pos.length_squared() < 1.0
    }
}

impl<C: MC<D>, const D: usize> VolumeQureyPosValue<C::Vector, C::Number, D> for CSGSphere<u8, C, D> {
    fn get_value(&self, pos: C::Vector) -> u8 {
        if self.is_position_valid(pos) {
            self.v
        } else {
            MATERIAL_ID_NONE
        }
    }
}

impl<C: MC<D>, const D: usize> VolumeQureyAABB<C::Vector, C::Number, D> for CSGSphere<u8, C, D> {
    fn get_aabb_value(&self, aabb: AABB<C::Vector, C::Number, D>) -> VolumeQureyAABBResult {
        let aabb: AABB<C::VectorF, f32, D> = aabb.mul_mat(&self.mat);

        let (min, max) = aabb.collides_unit_sphere();

        if max {
            VolumeQureyAABBResult::Full(self.v)
        } else if min {
            VolumeQureyAABBResult::Mixed
        } else {
            VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
        }
    }
}
