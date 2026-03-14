use std::marker::PhantomData;

use crate::{util::{aabb::AABB, matrix::Ma, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeGradient, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

pub mod r#box;
pub mod sphere;
pub mod all;
pub mod cylinder;

#[derive(Debug, Copy, Clone)]
pub struct CSGPrimitive<P: PrimitiveType, M, V: Ve<T, D>, T: Nu, const D: usize> {
    matrix: V::Matrix,
    inverse_matrix: V::Matrix,
    material: M,
    aabb: AABB<V, T, D>,
    needs_aabb_recompute: bool,
    primitive_type: PhantomData<P>,
}

#[derive(Copy, Clone, Debug)]
pub enum SampleAABBResult {
    Full,
    Empty,
    Mixed,
}

pub trait PrimitiveType: Copy {
    fn calculate_bounds<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix) -> AABB<V, T, D>;

    fn sample_pos<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix, pos: V) -> bool;
    
    fn sample_aabb<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix, aabb: AABB<V, T, D>) -> SampleAABBResult;
}

impl<P: PrimitiveType, M, V: Ve<T, D>, T: Nu, const D: usize> CSGPrimitive<P, M, V, T, D> {
    pub fn new(matrix: V::Matrix, material: M) -> Self {
        Self {
            inverse_matrix: matrix.inverse(),
            matrix,
            material,
            aabb: AABB::default(),
            needs_aabb_recompute: true,
            primitive_type: PhantomData,
        }
    }

    pub fn get_mat(&self) -> V::Matrix {
        self.matrix
    }

    pub fn get_mat_inv(&self) -> V::Matrix {
        self.inverse_matrix
    }

    pub fn set_mat(&mut self, mat: V::Matrix) {
        self.inverse_matrix = mat.inverse();
        self.matrix = mat;
        self.needs_aabb_recompute = true;
    }
}

impl<P: PrimitiveType, M, V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for CSGPrimitive<P, M, V, T, D> {
    fn calculate_bounds(&mut self) {
        if !self.needs_aabb_recompute {
            return;
        }

        self.aabb = P::calculate_bounds(&self.matrix, &self.inverse_matrix);
        self.needs_aabb_recompute = false;
    }

    fn get_bounds(&self) -> AABB<V, T, D> {
        self.aabb
    }
}

impl<P: PrimitiveType, M, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValid<V, T, D> for CSGPrimitive<P, M, V, T, D> {
    fn is_position_valid(&self, pos: V) -> bool {
        P::sample_pos(&self.matrix, &self.inverse_matrix, pos)
    }
}

impl<P: PrimitiveType, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValue<V, T, D> for CSGPrimitive<P, u8, V, T, D> {
    fn get_value(&self, pos: V) -> u8 {
        if P::sample_pos(&self.matrix, &self.inverse_matrix, pos) {
            self.material
        } else {
            MATERIAL_ID_NONE
        }
    }
}

impl<P: PrimitiveType, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyAABB<V, T, D> for CSGPrimitive<P, u8, V, T, D> {
    fn get_aabb_value(&self, aabb: AABB<V, T, D>) -> VolumeQureyAABBResult {
        match P::sample_aabb(&self.matrix, &self.inverse_matrix, aabb) {
            SampleAABBResult::Full => VolumeQureyAABBResult::Full(self.material),
            SampleAABBResult::Empty => VolumeQureyAABBResult::Full(MATERIAL_ID_NONE),
            SampleAABBResult::Mixed => VolumeQureyAABBResult::Mixed,
        }
    }
}

impl<P: PrimitiveType, M, V: Ve<T, D>, T: Nu, const D: usize> VolumeGradient<V::VectorF, D> for CSGPrimitive<P, M, V, T, D> {
    fn get_gradient_at_position(&self, pos: V::VectorF) -> V::VectorF {
        todo!()        
    }
}

