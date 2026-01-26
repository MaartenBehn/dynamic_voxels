use octa_force::glam::{vec3, IVec3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::{aabb::AABB, math_config::MC, matrix::Ma, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeGradient, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

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

/**
*          |
*  x---------------x
*  |       |       |
*  |       q --------> p
*  |       |       |
*  |       x       |
*  |       |       |
*  |       |       |
*  |       |       |
*  x---------------x
*          |
*
* From: https://github.com/MaartenBehn/distance3d/blob/master/distance3d/distance/_plane.py
*    t = np.dot(plane_normal, point - plane_point)
*    closest_point_plane = point - t * plane_normal
**/
pub fn get_gradient_of_unit_box(to_pos: Vec3) -> Vec3 {
    let normal = to_pos.signum();

    let t = normal.dot(to_pos);
    // let q = to_pos - t * normal;
    // let v = q - to_pos;
    
    -t * normal
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> VolumeGradient<V::VectorF, D> for CSGBox<M, V, T, D> {
    fn get_gradient_at_position(&self, pos: V::VectorF) -> V::VectorF {
        let to_pos = self.mat.mul_vector(pos);

        let normal = to_pos.signum();

        let t = normal.dot(to_pos);
        // let q = to_pos - t * normal;
        // let v = q - to_pos;

        normal * -t
    }
}
