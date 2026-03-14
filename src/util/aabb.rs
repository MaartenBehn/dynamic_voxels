use std::{iter, marker::PhantomData};

use itertools::Either;
use octa_force::glam::{IVec2, IVec3, Mat3, Mat4, Vec2, Vec3, Vec3A, ivec2, ivec3, vec2, vec3a, vec4};

use crate::util::vector::Ve;

use super::{matrix::Ma, number::Nu};

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

union AABBUnion<VA: Ve<T, DA>, VB: Ve<T, DB>, T: Nu, const DA: usize, const DB: usize> {
    a: AABB<VA, T, DA>,
    b: AABB<VB, T, DB>,
}

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
        self.size().max_value().0
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
            self.min[i] < other.max[i] && other.min[i] < self.max[i]
        })
    }

    pub fn contains_aabb(&self, other: Self) -> bool {
        (0..D).all(|i| {
            self.min[i] < other.min[i] && other.max[i] < self.max[i]
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
                let mat: Mat3 = mat.cast_into();
                let min: Vec2 = min.ve_into();
                let max: Vec2 = max.ve_into();

                let corners = [
                    vec3a(min[0],min[1], 1.0),
                    vec3a(min[0],max[1], 1.0),
                    vec3a(max[0],min[1], 1.0),
                    vec3a(max[0],max[1], 1.0),
                ];

                let mut min = vec3a(f32::INFINITY, f32::INFINITY, 1.0);
                let mut max = vec3a(f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);

                for corner in corners {
                    let transformed_corner = mat.mul_vec3a(corner);

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }

                Self::new(V::ve_from(min), V::ve_from(max))
            }
            3 => {
                let mat: Mat4 = mat.cast_into();
                let min: Vec3A = min.ve_into();
                let max: Vec3A = max.ve_into();

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
                    let transformed_corner = mat.mul_vec4(corner);

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }

                Self::new(V::ve_from(min), V::ve_from(max))
            }
            _ => unreachable!()
        }
    }
    
    pub fn get_sampled_positions(self, step: T) -> impl Iterator<Item = V> {
        if step <= T::ZERO || !self.valid() {
            return Either::Left(iter::empty())
        }

        let min = (self.min / step);
        let max = (self.max / step);

        match D {
            2 => {
                let min: IVec2 = min.ve_into();
                let max: IVec2 = max.ve_into();

                Either::Right(Either::Left((min.x..=max.x)
                    .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
                    .map(move |(x, y)| V::ve_from(ivec2(x, y)) * step)))
            }
            3 => {
                let min: IVec3 = min.ve_into();
                let max: IVec3 = max.ve_into();

                Either::Right(Either::Right((min.x..=max.x)
                    .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
                    .flat_map(move |(x, y)| iter::repeat((x, y)).zip(min.z..=max.z))
                    .map(move |((x, y), z)| V::ve_from(ivec3(x, y, z)) * step)))
            }
            _ => unreachable!()
        }
    }

    
    pub fn mul_mat<M: Ma<D>, VF: Ve<f32, D>>(&self, mat: &M) -> AABB<VF, f32, D> {
        match D {
            2 => todo!(),
            3 => {
                let min: Vec3 = self.min.ve_into();
                let max: Vec3 = self.max.ve_into();
                let mat: Mat4 = mat.cast_into();

                let center = (min + max) * 0.5;
                let extent = (max - min) * 0.5;

                let new_center = mat.transform_point3(center);

                let mx = mat.x_axis.truncate().abs();
                let my = mat.y_axis.truncate().abs();
                let mz = mat.z_axis.truncate().abs();

                let new_extent = Vec3::new(
                    mx.x * extent.x + my.x * extent.y + mz.x * extent.z,
                    mx.y * extent.x + my.y * extent.y + mz.y * extent.z,
                    mx.z * extent.x + my.z * extent.y + mz.z * extent.z,
                );

                AABB::new(VF::ve_from(new_center - new_extent), VF::ve_from(new_center + new_extent))
            },
            _ => unreachable!()
        }
    }

    pub fn to_f<V2: Ve<f32, D>>(self) -> AABB<V2, f32, D> {
        AABB::new(self.min.to_vecf(), self.max.to_vecf())
    }

    pub fn from_f<V2: Ve<f32, D>>(aabb: AABB<V2, f32, D>) -> Self {
        AABB::new(V::from_vecf(aabb.min()), V::from_vecf(aabb.max()))
    }

    pub fn to_f2d(self) -> AABB2 {
        match D {
            2 => {
                AABB2::new(self.min().ve_into(), self.max().ve_into())
            }
            _ => unreachable!()
        }
    }

    pub fn to_f3d(self) -> AABB3 {
        match D {
            3 => {
                AABB3::new(self.min().ve_into(), self.max().ve_into())
            }
            _ => unreachable!()
        }
    }

    pub fn to_i2d(self) -> IAABB2 {
        match D {
            2 => {
                IAABB2::new(self.min().ve_into(), self.max().ve_into())
            }
            _ => unreachable!()
        }
    }

    pub fn to_i3d(self) -> IAABB3 {
        match D {
            3 => {
                IAABB3::new(self.min().ve_into(), self.max().ve_into())
            }
            _ => unreachable!()
        }
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> Default for AABB<V, T, D> {
    fn default() -> Self {
        Self::new(V::MAX, V::MIN)
    }
}

