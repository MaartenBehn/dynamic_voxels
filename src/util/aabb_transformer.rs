use octa_force::glam::{Mat3, Mat4, Vec2};

use crate::util::{aabb::AABB, matrix::Ma, vector::{CastInto, Ve}};

#[derive(Debug, Copy, Clone)]
pub struct AABBTransformer<M: Ma<D>, V: Ve<f32, D>, const D: usize> {
    center_mat: M,
    abs: [V; D],
}

impl<M: Ma<D>, V: Ve<f32, D>, const D: usize> AABBTransformer<M, V, D> {
    pub fn new(mat: M) -> Self {

        let mut abs = [V::ZERO; D];
        for i in 0..D {
            abs[i] = V::new(mat.truc_col(i)).abs();
        }

        Self {
            center_mat: mat,
            abs,    
        }
    }

    #[inline(always)]
    pub fn transform_pos(&self, pos: V) -> V {
        self.center_mat.mul_vector(pos)
    }

    #[inline(always)]
    pub fn transform_aabb(&self, aabb: AABB<V, f32, D>) -> AABB<V, f32, D> {
        let center = (aabb.min() + aabb.max()) * 0.5;
        let extent = (aabb.max() - aabb.min()) * 0.5;
 
        let new_center = self.center_mat.mul_vector(center);

        let mut arr = [0.0; D];
        for i in 0..D {
            arr[i] = self.abs[i].dot(extent); 
        }
        let new_extent = V::new(arr);

        AABB::new(new_center - new_extent, new_center + new_extent)
    }
}
