use octa_force::glam::{vec3, IVec3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::{aabb::{AABB}, math_config::{MC}, matrix::Ma, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::Base;

#[derive(Clone, Copy, Debug)]
pub struct CSGBox<M, V: Ve<T, D>, T: Nu, const D: usize> {
    mat: V::Matrix,
    v: M
}

impl<M: Base, V: Ve<T, D>, T: Nu, const D: usize> CSGBox<M, V, T, D> {
    pub fn new(pos: V::VectorF, size: V::VectorF, mat: M) -> Self {
        CSGBox {
            mat: V::Matrix::from_scale_translation(size, pos).inverse(),
            v: mat,
        }
    }
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for CSGBox<M, V, T, D> {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<V, T, D> {
        let mat = self.mat.inverse();
        AABB::from_box(&mat)
    }
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValid<V, T, D> for CSGBox<M, V, T, D> {
    fn is_position_valid(&self, pos: V) -> bool {
        let pos = self.mat.mul_vector(V::to_vector_f(pos));

        let aabb = AABB::<V::VectorF, f32, D>::new(V::VectorF::new([-0.5; D]), V::VectorF::new([0.5; D]));

        aabb.pos_in_aabb(pos)
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValue<V, T, D> for CSGBox<u8, V, T, D> {
    fn get_value(&self, pos: V) -> u8 {
        if self.is_position_valid(pos) {
            self.v
        } else {
            MATERIAL_ID_NONE
        }
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyAABB<V, T, D> for CSGBox<u8, V, T, D> {
    fn get_aabb_value(&self, aabb: AABB<V, T, D>) -> VolumeQureyAABBResult {
        let aabb = aabb.mul_mat(&self.mat);

        let b = AABB::<V::VectorF, f32, D>::new(V::VectorF::new([-0.5; D]), V::VectorF::new([0.5; D]));


        if aabb.contains_aabb(b) {
            VolumeQureyAABBResult::Full(self.v)
        } else if aabb.collides_aabb(b) {
            VolumeQureyAABBResult::Mixed
        } else {
            VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
        }
    }
}

/*
#[derive(Clone, Copy, Debug)]
pub struct CSGBox<T> {
    mat: Mat4,
    v: T
}

impl<T: Base> CSGBox<T> {
    pub fn new(mat: Mat4) -> Self {
        CSGBox {
            mat: mat.inverse(),
            v: T::base(),
        }
    }
}

impl<T> VolumeBounds for CSGBox<T> {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB3 {
        let mat = self.mat.inverse();
        AABB3::from_box(&mat)
    }
}

impl<T> VolumeBoundsI for CSGBox<T> {
    fn calculate_bounds_i(&mut self) {}
    fn get_bounds_i(&self) -> AABBI { self.get_bounds().into() }
}

impl<T> VolumeQureyPosValid for CSGBox<T> {
    fn is_position_valid_vec3(&self, pos: Vec3A) -> bool {
        let pos = self.mat.mul_vec4(Vec4::from((pos, 1.0)));
        let aabb = AABB3::new(
            vec3(-0.5, -0.5, -0.5), 
            vec3(0.5, 0.5, 0.5));

        aabb.pos_in_aabb(pos)
    }
}

impl VolumeQureyPosValue for CSGBox<u8> {
    fn get_value(&self, pos: Vec3A) -> u8 {
        if self.is_position_valid_vec3(pos) {
            self.v
        } else {
            MATERIAL_ID_NONE
        }
    }
}

impl VolumeQureyPosValueI for CSGBox<u8> {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        self.get_value(pos.as_vec3a())
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        unimplemented!()
    }
}

impl VolumeQureyAABB for CSGBox<u8> {
    fn get_aabb_value(&self, aabb: AABB3) -> VolumeQureyAABBResult {
        let aabb = aabb.mul_mat(&self.mat);

        let b = AABB3::new(
            vec3(-0.5, -0.5, -0.5), 
            vec3(0.5, 0.5, 0.5));

        if aabb.contains_aabb(b) {
            VolumeQureyAABBResult::Full(self.v)
        } else if aabb.collides_aabb(b) {
            VolumeQureyAABBResult::Mixed
        } else {
            VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
        }

    }
}

impl VolumeQureyAABBI for CSGBox<u8> {
    fn get_aabb_value_i(&self, aabb: AABBI) -> VolumeQureyAABBResult {
        self.get_aabb_value(aabb.into())
    }
}
*/
