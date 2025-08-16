use std::{fmt::Debug, ops::Mul};

use octa_force::glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec3A, Vec4};

use super::vector::Ve;

pub trait Ma<const D: usize>:
    Sized
    + Copy
    + Debug
    + Default
    + Send
    + Sync
    + Mul<Output = Self>
{
    fn index(&self, i: usize, j: usize) -> f32;
    fn from_scale<V: Ve<f32, D>>(scale: V) -> Self;
    fn from_scale_translation<V: Ve<f32, D>>(scale: V, translation: V) -> Self;

    fn inverse(&self) -> Self;
    fn mul_vector<V: Ve<f32, D>>(&self, v: V) -> V;

    fn to_mat3(self) -> Mat3;
    fn to_mat4(self) -> Mat4;
}

impl Ma<3> for Mat4 {
    fn index(&self, i: usize, j: usize) -> f32 {
        self.col(i)[j]
    }

    fn from_scale<V: Ve<f32, 3>>(scale: V) -> Self {
        Mat4::from_scale_rotation_translation(scale.to_vec3(), Quat::IDENTITY, Vec3::ZERO)
    }

    fn from_scale_translation<V: Ve<f32, 3>>(scale: V, translation: V) -> Self {
        Mat4::from_scale_rotation_translation(scale.to_vec3(), Quat::IDENTITY, translation.to_vec3())
    }

    fn inverse(&self) -> Self { Mat4::inverse(&self) }

    fn mul_vector<V: Ve<f32, 3>>(&self, v: V) -> V { V::from_vec4h(self.mul_vec4(Vec4::from((v.to_vec3a(), 1.0)))) }

    fn to_mat3(self) -> Mat3 { unreachable!() }
    fn to_mat4(self) -> Mat4 { self }
}


impl Ma<2> for Mat3 {
    fn index(&self, i: usize, j: usize) -> f32 {
        self.col(i)[j]
    }

    fn from_scale<V: Ve<f32, 2>>(scale: V) -> Self {
        Mat3::from_scale_angle_translation(scale.to_vec2(), 0.0, Vec2::ZERO)
    }

    fn from_scale_translation<V: Ve<f32, 2>>(scale: V, translation: V) -> Self {
        Mat3::from_scale_angle_translation(scale.to_vec2(), 0.0, translation.to_vec2())
    }

    fn inverse(&self) -> Self { Mat3::inverse(&self) }

    fn mul_vector<V: Ve<f32, 2>>(&self, v: V) -> V { V::from_vec3a(self.mul_vec3a(Vec3A::from((v.to_vec2(), 1.0)))) }

    fn to_mat3(self) -> Mat3 { self }
    fn to_mat4(self) -> Mat4 { unreachable!() }
}

