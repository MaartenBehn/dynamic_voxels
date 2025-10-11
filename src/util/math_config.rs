use std::{fmt::Debug, marker::PhantomData, usize};

use octa_force::glam::{IVec2, IVec3, Mat3, Mat4, Vec2, Vec3, Vec3A};

use super::{aabb::AABB, matrix::Ma, number::Nu, vector::Ve};

pub trait MC<V: Ve<T, D>, T: Nu, const D: usize>: Copy + Clone + Default + Debug {
    type Matrix: Ma<D>;
    type VectorF: Ve<f32, D>;

    fn to_vector(v: Self::VectorF) -> V;
    fn to_vector_f(v: V) -> Self::VectorF;
}
