use octa_force::{egui::Vec2, glam::{IVec3, Mat3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4, vec3, vec3a, vec4}};

use crate::{csg::primitves::{CSGPrimitive, PrimitiveType}, util::{aabb::AABB, matrix::Ma, number::Nu, vector::{CastInto, Ve}}};

#[derive(Clone, Copy, Debug)]
pub struct CSGBox {}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> CSGPrimitive<CSGBox, M, V, T, D> {
    pub fn new_box(pos: V::VectorF, size: V::VectorF, mat: M) -> Self {
        CSGPrimitive::new(V::Matrix::from_scale_translation(size, pos), mat)
    }
}

impl PrimitiveType for CSGBox {
    fn calculate_bounds<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix) -> AABB<V, T, D> {
        match D {
            2 => {
                let min = Vec2::splat(-0.5);
                let max = Vec2::splat(0.5);

                let corners = [
                    vec3a(min[0],min[1], 1.0),
                    vec3a(min[0],max[1], 1.0),
                    vec3a(max[0],min[1], 1.0),
                    vec3a(max[0],max[1], 1.0),
                ];

                let mut min = vec3a(f32::INFINITY, f32::INFINITY, 1.0);
                let mut max = vec3a(f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);

                for corner in corners {
                    let mat: Mat3 = mat.cast_into();
                    let transformed_corner = mat.mul_vec3a(corner);

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }
 
                AABB::new(V::ve_from(min), V::ve_from(max))
            }
            3 => {
                let min = Vec3::splat(-0.5);
                let max = Vec3::splat(0.5);

                let corners = [
                    vec4(min[0],min[1], min[2], 1.0),
                    vec4(min[0],min[1], max[2], 1.0),
                    vec4(min[0],max[1], min[2], 1.0),
                    vec4(min[0],max[1], max[2], 1.0),
                    vec4(max[0],min[1], min[2], 1.0),
                    vec4(max[0],min[1], max[2], 1.0),
                    vec4(max[0],max[1], min[2], 1.0),
                    vec4(max[0],max[1], max[2], 1.0),
                ];

                let mut min = vec4(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);
                let mut max = vec4(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);

                for corner in corners {
                    let mat: Mat4 = mat.cast_into();
                    let transformed_corner = mat.mul_vec4(corner);

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }
 
                AABB::new(V::ve_from(min), V::ve_from(max))
            }
            _ => unreachable!()
        }
    }

    fn sample_pos<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix, pos: V) -> bool {
        let pos = mat.mul_vector(V::to_vector_f(pos));

        let aabb = AABB::<V::VectorF, f32, D>::new(
            V::VectorF::new([-0.5; D]), 
            V::VectorF::new([0.5; D]));

        aabb.pos_in_aabb(pos)
    }

    fn sample_aabb<V: Ve<T, D>, T: Nu, const D: usize>(mat: &V::Matrix, inv_mat: &V::Matrix, aabb: AABB<V, T, D>) -> super::SampleAABBResult {
        let aabb = aabb.mul_mat(mat);

        let b = AABB::<V::VectorF, f32, D>::new(
            V::VectorF::new([-0.5; D]), 
            V::VectorF::new([0.5; D]));

        if b.contains_aabb(aabb) {
            super::SampleAABBResult::Full
        } else if b.collides_aabb(aabb) {
            super::SampleAABBResult::Mixed
        } else {
            super::SampleAABBResult::Empty
        }
    }
}


/*


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
*/
