use std::marker::PhantomData;

use octa_force::glam::{vec3, IVec3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::{aabb::{AABB}, math_config::MC, matrix::Ma, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{Base};

#[derive(Clone, Copy, Debug, Default)]
pub struct CSGSphere<M, V: Ve<T, D>, T: Nu, const D: usize> {
    mat: V::Matrix,
    m: M,
    p: PhantomData<T>
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> CSGSphere<M, V, T, D> {
    pub fn new_sphere(center: V::VectorF, radius: f32, mat: M) -> Self {
        CSGSphere {
            mat: V::Matrix::from_scale_translation(
                V::VectorF::ONE * radius,
                center,
            ).inverse(),
            m: mat,
            p: Default::default(),
        }
    }
}

impl<M: Base, V: Ve<T, 3>, T: Nu> CSGSphere<M, V, T, 3> {
    pub fn new_disk(center: V::VectorF, radius: f32, height: f32, mat: M) -> Self {
        CSGSphere {
            mat: V::Matrix::from_scale_translation(
                V::VectorF::new([radius, radius, height]),
                center,
            ).inverse(),
            m: mat,
            p: Default::default(),
        }
    }
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for CSGSphere<M, V, T, D> {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<V, T, D> {
        let mat = self.mat.inverse();
        AABB::from_sphere(&mat)
    }
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValid<V, T, D> for CSGSphere<M, V, T, D> {
    fn is_position_valid(&self, pos: V) -> bool {
        let pos = self.mat.mul_vector(V::to_vector_f(pos));
        pos.length_squared() < 1.0
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValue<V, T, D> for CSGSphere<u8, V, T, D> {
    fn get_value(&self, pos: V) -> u8 {
        if self.is_position_valid(pos) {
            self.m
        } else {
            MATERIAL_ID_NONE
        }
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyAABB<V, T, D> for CSGSphere<u8, V, T, D> {
    fn get_aabb_value(&self, aabb: AABB<V, T, D>) -> VolumeQureyAABBResult {
        let aabb: AABB<V::VectorF, f32, D> = aabb.mul_mat(&self.mat);

        let (min, max) = aabb.collides_unit_sphere();

        if max {
            VolumeQureyAABBResult::Full(self.m)
        } else if min {
            VolumeQureyAABBResult::Mixed
        } else {
            VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
        }
    }
}
