use octa_force::glam::{IVec3, Mat3, Mat4, Quat, UVec3, Vec2, Vec3, Vec3A, Vec4, vec3, vec3a, vec4};

use crate::{csg::primitves::{CSGPrimitive, PrimitiveType}, util::{aabb::AABB, matrix::Ma, number::Nu, vector::{CastInto, IntoT, Ve}}};
use crate::util::vector::CastFrom;   

#[derive(Clone, Copy, Debug, Default)]
pub struct CSGCylinder {}

impl<M, V: Ve<T, 3>, T: Nu> CSGPrimitive<CSGCylinder, M, V, T, 3> {
    pub fn new_cylinder_from_a_to_b(a: V::VectorF, b: V::VectorF, r: f32, mat: M) -> Self {
         CSGPrimitive::new( cylinder_between::<V, _>(a, b, r), mat)
    }
}

pub fn cylinder_between<V: Ve<T, 3>, T: Nu>(a: V::VectorF, b: V::VectorF, r: f32) -> V::Matrix {
    let axis = b - a;
    let len = axis.length();
    let dir = axis / len;

    let center = (a + b) * 0.5;

    let rot = Quat::from_rotation_arc(Vec3::Z, dir.ve_into());

    V::Matrix::cast_from(Mat4::from_translation(center.ve_into())
        * Mat4::from_quat(rot)
        * Mat4::from_scale(Vec3::new(r, r, len * 0.5)))
}

impl PrimitiveType for CSGCylinder {
    fn calculate_bounds<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix) -> AABB<V, T, D> {
        match D {
            2 => {
                let mat: Mat3 = mat.cast_into();
                let center = mat.z_axis.truncate();

                let l0 = mat.x_axis.truncate();
                let l1 = mat.y_axis.truncate();

                let extent = Vec2::new(
                    l0.x.abs() + l1.x.abs(),
                    l0.y.abs() + l1.y.abs(),
                );

                AABB::new(V::ve_from(center - extent),  V::ve_from(center + extent))
            }
            3 => {
                let mat: Mat4 = mat.cast_into();
                let center = mat.w_axis.truncate();

                let l0 = mat.x_axis.truncate();
                let l1 = mat.y_axis.truncate();
                let l2 = mat.z_axis.truncate();

                let extent = Vec3::new(
                    (l0.x * l0.x + l1.x * l1.x).sqrt() + l2.x.abs(),
                    (l0.y * l0.y + l1.y * l1.y).sqrt() + l2.y.abs(),
                    (l0.z * l0.z + l1.z * l1.z).sqrt() + l2.z.abs(),
                );

                AABB::new(V::ve_from(center - extent),  V::ve_from(center + extent))
            }
            _ => unreachable!()
        }
    }

    fn sample_pos<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix, pos: V) -> bool {
        let pos = inv_mat.mul_vector(V::to_vector_f(pos));
      
        let arr = pos.to_array();

        // height check
        if arr[D-1] < -1.0 || arr[D-1] > 1.0 {
            return false;
        }

        // radial check
        let radial: f32 = arr[..(D-1)].into_iter()
            .copied()
            .map(|i| i * i )
            .sum();
                
        radial <= 1.0
    }

    fn sample_aabb<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix, aabb: AABB<V, T, D>) -> super::SampleAABBResult {
        let aabb: AABB<V::VectorF, f32, D> = aabb.mul_mat(inv_mat);
       
        let min = aabb.min().to_array();
        let max = aabb.max().to_array();
        // height check
        if max[D-1] < -1.0 || min[D-1] > 1.0 {
            return super::SampleAABBResult::Empty;
        }
       
        // --- Circle vs AABB test
        let radial: f32 = (0..(D-1))
            .map(|i| { 
                let d = if 0.0 < min[i] {
                    min[i]
                } else if 0.0 > max[i] {
                    max[i]
                } else {
                    0.0
                };
                d * d 
            }).sum();

        if radial > 1.0 {
            return super::SampleAABBResult::Mixed;
        }

        // --- Full containment test
        if min[D-1] >= -1.0 && max[D-1] <= 1.0 {
            let radial: f32 = (0..(D-1))
                .map(|i| (min[i] * min[i]).max(max[i] * max[i]) )
                .sum();

            if radial <= 1.0 {
                return super::SampleAABBResult::Full;
            }
        }

        super::SampleAABBResult::Mixed
    }
}

