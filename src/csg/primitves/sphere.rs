use octa_force::glam::{IVec3, Mat3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4, vec2, vec3, vec3a, vec4};

use crate::{csg::primitves::{CSGPrimitive, PrimitiveType}, util::{aabb::AABB, matrix::Ma, number::Nu, vector::{CastInto, Ve}}};

#[derive(Clone, Copy, Debug, Default)]
pub struct CSGSphere {}


impl<M, V: Ve<f32, D>, const D: usize> CSGPrimitive<CSGSphere, M, V, D> {
    pub fn new_sphere(center: V, radius: f32, mat: M) -> Self {
        CSGPrimitive::new(V::Matrix::from_scale_translation(
                V::ONE * radius,
                center,
            ), mat)
    }
}

impl<M, V: Ve<f32, 3>> CSGPrimitive<CSGSphere, M, V, 3> {
    pub fn new_disk(center: V, radius: f32, height: f32, mat: M) -> Self {
        CSGPrimitive::new( V::Matrix::from_scale_translation(
                V::new([radius, radius, height]),
                center,
            ), mat)
    }
}

impl PrimitiveType for CSGSphere {
    fn calculate_bounds<V: Ve<f32, D>, const D: usize>(mat: &V::Matrix) -> AABB<V, f32, D> {
        match D {
            2 => {
                let mat: Mat3 = mat.cast_into();
                let center = vec2(mat.z_axis.x, mat.z_axis.y);
                let extend = vec2(
                    f32::sqrt(mat.x_axis.x * mat.x_axis.x + mat.x_axis.y * mat.x_axis.y + mat.x_axis.z * mat.x_axis.z),
                    f32::sqrt(mat.y_axis.x * mat.y_axis.x + mat.y_axis.y * mat.y_axis.y + mat.y_axis.z * mat.y_axis.z),
                );

                AABB::new(V::ve_from(center - extend), V::ve_from(center + extend))
            }
            3 => {
                let mat: Mat4 = mat.cast_into();
                
                let center = vec3a(mat.w_axis.x, mat.w_axis.y, mat.w_axis.z);
                let extend = vec3a(
                    f32::sqrt(mat.x_axis.x * mat.x_axis.x + mat.x_axis.y * mat.x_axis.y + mat.x_axis.z * mat.x_axis.z),
                    f32::sqrt(mat.y_axis.x * mat.y_axis.x + mat.y_axis.y * mat.y_axis.y + mat.y_axis.z * mat.y_axis.z),
                    f32::sqrt(mat.z_axis.x * mat.z_axis.x + mat.z_axis.y * mat.z_axis.y + mat.z_axis.z * mat.z_axis.z),
                );

                AABB::new(V::ve_from(center - extend), V::ve_from(center + extend))
            }
            _ => unreachable!()
        }
    }

    fn sample_pos<V: Ve<f32, D>, const D: usize>(pos: V) -> bool {
        pos.length_squared() < 1.0
    }

    fn sample_aabb<V: Ve<f32, D>, const D: usize>(aabb: AABB<V, f32, D>) -> super::SampleAABBResult {
        let a = aabb.min() * aabb.min();
        let b = aabb.max() * aabb.max();
        let dmax = a.max(b).element_sum();
        let dmin = (V::ZERO.lt(aabb.min()) * a + V::ZERO.gt(aabb.max()) * b).element_sum();

        if dmax <= 1.0 {
            super::SampleAABBResult::Full
        } else if dmin <= 1.0 {
            super::SampleAABBResult::Mixed
        } else {
            super::SampleAABBResult::Empty
        }
    }
}


/*
impl<M, V: Ve<T, D>, T: Nu, const D: usize> VolumeGradient<V::VectorF, D> for CSGSphere<M, V, T, D> {
    fn get_gradient_at_position(&self, pos: V::VectorF) -> V::VectorF {
        self.mat.mul_vector(pos)
    }
}
*/
