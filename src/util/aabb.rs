use std::{iter, marker::PhantomData};

use itertools::Either;
use octa_force::glam::{ivec2, ivec3, vec2, vec3a, vec4, IVec2, IVec3, Vec2, Vec3, Vec3A};

use super::{math::{nalgebra_point_to_vec3a, vec3a_to_nalgebra_point}, matrix::Ma, number::Nu, vector::{nalgebra_to_vector, vector_to_nalgebra, Ve}};

#[derive(Clone, Copy, Debug)]
pub struct AABB<V: Ve<T, D>, T: Nu, const D: usize> {
    min: V,
    max: V,
    p: PhantomData<T>,
}

pub type AABB3 = AABB<Vec3A, f32, 3>;
pub type AABB2 = AABB<Vec2, f32, 2>;
pub type IAABB3 = AABB<IVec3, i32, 3>;
pub type IAABB2 = AABB<IVec2, i32, 2>;

impl<V: Ve<T, D>, T: Nu, const D: usize> AABB<V, T, D>  {
    pub fn new(min: V, max: V) -> Self {
        Self { min, max, p: Default::default() }
    }

    pub fn min(&self) -> V {
        self.min
    }

    pub fn max(&self) -> V {
        self.max
    }

    pub fn size(&self) -> V {
        if self.valid() {
            self.max - self.min
        } else {
            V::ZERO
        }
    }

    pub fn center(&self) -> V {
        (self.max + self.min) / T::TWO
    }

    pub fn infinte() -> Self {
        Self::new(V::MIN, V::MAX)
    }

    pub fn union(self, other: Self) -> Self {
        Self::new(self.min.min(other.min), self.max.max(other.max))
    }

    pub fn union_mut(&mut self, other: Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn intersect(self, other: Self) -> Self {
        Self::new(self.min.max(other.min), self.max.min(other.max))
    }

    pub fn intersect_mut(&mut self, other: Self) {
        self.min = self.min.max(other.min);
        self.max = self.max.min(other.max);
    }

    pub fn union_point(self, point: V) -> Self {
        Self::new(self.min.min(point), self.max.max(point))
    }

    pub fn union_point_mut(&mut self, point: V) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn largest_axis(self) -> usize {
        self.size().max_index()
    }
    
    pub fn pos_in_aabb(self, pos: V) -> bool {
        (0..D).all(|i| {
            self.min[i] <= pos[i] && pos[i] <= self.max[i]
        })
    }

    pub fn surface_area(self) -> T {
        let size = self.size();
        T::TWO * size.dot(size)
    }

    pub fn collides_aabb(&self, other: Self) -> bool {
        (0..D).all(|i| {
            self.min[i] <= other.max[i] && other.min[i] <= self.max[i]
            // self.min.x <= other.max.x && other.min.x <= self.max.x
        })
    }

    pub fn contains_aabb(&self, other: Self) -> bool {
        (0..D).all(|i| {
            self.min[i] <= other.min[i] && other.max[i] <= self.max[i]
            // self.min.x <= other.max.x && other.min.x <= self.max.x
        })
    }

    pub fn valid(self) -> bool {
        (0..D).all(|i| {
            self.min[i] < self.max[i]
        }) 
    }

    pub fn from_min_max<M: Ma<D>>(mat: &M, min: V, max: V) -> Self { 
        match D {
            2 => {
                let min = min.to_vec2();
                let max = max.to_vec2();

                let corners = [
                    vec3a(min[0],min[1], 1.0),
                    vec3a(min[0],max[1], 1.0),
                    vec3a(max[0],min[1], 1.0),
                    vec3a(max[0],max[1], 1.0),
                ];

                let mut min = vec3a(f32::INFINITY, f32::INFINITY, 1.0);
                let mut max = vec3a(f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);

                for corner in corners {
                    let transformed_corner = mat.to_mat3().mul_vec3a(corner);

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }

                Self::new(V::from_vec3a(min), V::from_vec3a(max))
            }
            3 => {
                let min = min.to_vec3a();
                let max = max.to_vec3a();

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
                    let transformed_corner = mat.to_mat4().mul_vec4(corner);

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }

                Self::new(V::from_vec4h(min), V::from_vec4h(max))
            }
            _ => unreachable!()
        }
    }

    pub fn from_box<M: Ma<D>>(mat: &M) -> Self {
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
                    let transformed_corner = mat.to_mat3().mul_vec3a(corner);

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }
 
                Self::new(V::from_vec3a(min), V::from_vec3a(max))
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
                    let transformed_corner = mat.to_mat4().mul_vec4(corner);

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }
 
                Self::new(V::from_vec4h(min), V::from_vec4h(max))
            }
            _ => unreachable!()
        }
    }

    pub fn from_sphere<M: Ma<D>>(mat: &M) -> Self {
        match D {
            2 => {
                let mat = mat.to_mat3();
                let a = vec3a(
                    f32::sqrt(mat.x_axis.x.powf(2.0) + mat.x_axis.y.powf(2.0) + mat.x_axis.z.powf(2.0)),
                    f32::sqrt(mat.y_axis.x.powf(2.0) + mat.y_axis.y.powf(2.0) + mat.y_axis.z.powf(2.0)),
                    0.0
                );
                let b = vec3a(mat.z_axis.x, mat.z_axis.y, 1.0);

                Self::new(V::from_vec3a(b - a), V::from_vec3a(b + a))
            }
            3 => {
                let mat = mat.to_mat4();
                let a = vec4(
                    f32::sqrt(mat.x_axis.x.powf(2.0) + mat.x_axis.y.powf(2.0) + mat.x_axis.z.powf(2.0)),
                    f32::sqrt(mat.y_axis.x.powf(2.0) + mat.y_axis.y.powf(2.0) + mat.y_axis.z.powf(2.0)),
                    f32::sqrt(mat.z_axis.x.powf(2.0) + mat.z_axis.y.powf(2.0) + mat.z_axis.z.powf(2.0)),
                    0.0
                );
                let b = vec4(mat.w_axis.x, mat.w_axis.y, mat.w_axis.z, 1.0);

                Self::new(V::from_vec4h(b - a), V::from_vec4h(b + a))
            }
            _ => unreachable!()
        }
    }

    pub fn collides_unit_sphere(self) -> (bool, bool) {

        let a = self.min * self.min;
        let b = self.max * self.max;
        let dmax = a.max(b).element_sum();
        let dmin = 
            (V::ZERO.lt(self.min) * a 
           + V::ZERO.gt(self.max) * b).element_sum();

        let min = dmin <= T::ONE;
        let max = dmax <= T::ONE;

        (min, max)
    }

    pub fn get_sampled_positions(self, step: T) -> impl Iterator<Item = V> {
        if step <= T::ZERO || !self.valid() {
            return Either::Left(iter::empty())
        }

        let min = (self.min / step);
        let max = (self.max / step);

        match D {
            2 => {
                let min = min.to_ivec2();
                let max = max.to_ivec2();

                Either::Right(Either::Left((min.x..=max.y)
                    .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
                    .map(move |(x, y)| V::from_ivec2(ivec2(x, y)) * step)))
            }
            3 => {
                let min = min.to_ivec3();
                let max = max.to_ivec3();

                Either::Right(Either::Right((min.x..=max.y)
                    .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
                    .flat_map(move |(x, y)| iter::repeat((x, y)).zip(min.z..=max.z))
                    .map(move |((x, y), z)| V::from_ivec3(ivec3(x, y, z)) * step)))
            }
            _ => unreachable!()
        }
    }

    
    pub fn mul_mat<M: Ma<D>, VF: Ve<f32, D>>(&self, mat: &M) -> AABB<VF, f32, D> {
        AABB::new(mat.mul_vector(self.min.to_vecf()), mat.mul_vector(self.max.to_vecf()))
    }

    pub fn to_f<V2: Ve<f32, D>>(self) -> AABB<V2, f32, D> {
        AABB::new(self.min.to_vecf(), self.max.to_vecf())
    }

    pub fn from_f<V2: Ve<f32, D>>(aabb: AABB<V2, f32, D>) -> Self {
        AABB::new(V::from_vecf(aabb.min()), V::from_vecf(aabb.max()))
    } 
}

impl<V: Ve<T, D>, T: Nu, const D: usize> Default for AABB<V, T, D> {
    fn default() -> Self {
        Self::new(V::MAX, V::MIN)
    }
}

impl<V: Ve<f32, D>, const D: usize> From<&bvh::aabb::Aabb<f32, D>> for AABB<V, f32, D> {
    fn from(value: &bvh::aabb::Aabb<f32, D>) -> Self {
        AABB::new(nalgebra_to_vector(value.min), nalgebra_to_vector(value.max))
    }
}

impl<V: Ve<f32, D>, const D: usize> Into<bvh::aabb::Aabb<f32, D>> for AABB<V, f32, D> {
    fn into(self) -> bvh::aabb::Aabb<f32, D> {
        bvh::aabb::Aabb { min: vector_to_nalgebra(self.min), max: vector_to_nalgebra(self.max) }
    }
}
