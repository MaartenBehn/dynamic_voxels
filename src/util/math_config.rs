use std::usize;

use octa_force::glam::{IVec3, Mat3, Mat4, Vec2, Vec3, Vec3A};

use crate::model::generation::traits;

use super::{aabb::AABB, aabb3d::AABB3, matrix::Ma, number::Nu, vector::Ve};

pub trait MC<const D: usize> {
    type Matrix: Ma<D>;
    type Vector: Ve<Self::Number, D>;
    type VectorF: Ve<f32, D>;
    type Number: Nu;

    fn to_vector(v: Self::VectorF) -> Self::Vector;
    fn to_vector_f(v: Self::Vector) -> Self::VectorF;
}

#[derive(Debug, Clone, Copy)]
pub struct Float<const D: usize> {}
pub type Float3D = Float<3>;
impl MC<3> for Float<3> {
    type Matrix = Mat4;
    type Vector = Vec3A;
    type VectorF = Vec3A;
    type Number = f32;

    fn to_vector(v: Self::VectorF) -> Self::Vector { v }
    fn to_vector_f(v: Self::Vector) -> Self::VectorF { v }
}

pub type Float2D = Float<2>;
impl MC<2> for Float<2> {
    type Matrix = Mat3;
    type Vector = Vec2;
    type VectorF = Vec2;
    type Number = f32;

    fn to_vector(v: Self::VectorF) -> Self::Vector { v }
    fn to_vector_f(v: Self::Vector) -> Self::VectorF { v }
}

#[derive(Debug, Clone, Copy)]
pub struct Int<const D: usize>{}
pub type Int3D = Int<3>;
impl MC<3> for Int<3> {
    type Matrix = Mat4;
    type Vector = IVec3;
    type VectorF = Vec3A;
    type Number = i32;

    fn to_vector(v: Self::VectorF) -> Self::Vector { IVec3::from(v) }
    fn to_vector_f(v: Self::Vector) -> Self::VectorF { Vec3A::from(v) }
}
