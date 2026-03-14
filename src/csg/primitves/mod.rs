use std::marker::PhantomData;

use crate::{util::{aabb::AABB, aabb_transformer::AABBTransformer, matrix::Ma, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeGradient, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

pub mod r#box;
pub mod sphere;
pub mod all;
pub mod cylinder;

#[derive(Debug, Copy, Clone)]
pub struct CSGPrimitive<P: PrimitiveType, M, V: Ve<f32, D>, const D: usize> {
    matrix: V::Matrix,
    inverse_transfomer: AABBTransformer<V::Matrix, V, D>,
    material: M,
    aabb: AABB<V, f32, D>,
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
    fn calculate_bounds<V: Ve<f32, D>, const D: usize>(mat: &V::Matrix) -> AABB<V, f32, D>;

    fn sample_pos<V: Ve<f32, D>, const D: usize>(pos: V) -> bool;
    
    fn sample_aabb<V: Ve<f32, D>, const D: usize>(aabb: AABB<V, f32, D>) -> SampleAABBResult;
}

impl<P: PrimitiveType, M, V: Ve<f32, D>, const D: usize> CSGPrimitive<P, M, V, D> {
    pub fn new(matrix: V::Matrix, material: M) -> Self {
        Self {
            inverse_transfomer: AABBTransformer::new(matrix.inverse()),
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

    pub fn set_mat(&mut self, mat: V::Matrix) {
        self.inverse_transfomer = AABBTransformer::new(mat.inverse());
        self.matrix = mat;
        self.needs_aabb_recompute = true;
    }
}

impl<P: PrimitiveType, M, V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for CSGPrimitive<P, M, V::VectorF, D> {
    fn calculate_bounds(&mut self) {
        if !self.needs_aabb_recompute {
            return;
        }

        self.aabb = P::calculate_bounds(&self.matrix);
        self.needs_aabb_recompute = false;
    }

    fn get_bounds(&self) -> AABB<V, T, D> {
        AABB::from_f(self.aabb)
    }
}

impl<P: PrimitiveType, M, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValid<V, T, D> for CSGPrimitive<P, M, V::VectorF, D> {
    fn is_position_valid(&self, pos: V) -> bool {
        let pos = self.inverse_transfomer.transform_pos(pos.to_vecf());

        P::sample_pos(pos)
    }
}

impl<P: PrimitiveType, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValue<V, T, D> for CSGPrimitive<P, u8, V::VectorF, D> {
    fn get_value(&self, pos: V) -> u8 {
        let pos = self.inverse_transfomer.transform_pos(pos.to_vecf());

        if P::sample_pos(pos) {
            self.material
        } else {
            MATERIAL_ID_NONE
        }
    }
}

impl<P: PrimitiveType, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyAABB<V, T, D> for CSGPrimitive<P, u8, V::VectorF, D> {
    fn get_aabb_value(&self, aabb: AABB<V, T, D>) -> VolumeQureyAABBResult {
        let aabb = self.inverse_transfomer.transform_aabb(aabb.to_f());
        match P::sample_aabb(aabb) {
            SampleAABBResult::Full => VolumeQureyAABBResult::Full(self.material),
            SampleAABBResult::Empty => VolumeQureyAABBResult::Full(MATERIAL_ID_NONE),
            SampleAABBResult::Mixed => VolumeQureyAABBResult::Mixed,
        }
    }
}

impl<P: PrimitiveType, M, V: Ve<f32, D>, const D: usize> VolumeGradient<V, D> for CSGPrimitive<P, M, V, D> {
    fn get_gradient_at_position(&self, pos: V) -> V {
        todo!()        
    }
}

