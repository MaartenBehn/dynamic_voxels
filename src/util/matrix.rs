use std::{fmt::Debug, ops::Mul};

use octa_force::glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec3A, Vec4};

use crate::util::vector::{CastFrom, CastInto};

use super::vector::Ve;

pub trait Ma<const D: usize>:
    Sized
    + Copy
    + Debug
    + Default
    + Send
    + Sync
    + Mul<Output = Self>
    + CastFrom<Mat3>
    + CastFrom<Mat4>
    + CastInto<Mat3>
    + CastInto<Mat4>
{
    fn index(&self, i: usize, j: usize) -> f32;
    fn from_scale<V: Ve<f32, D>>(scale: V) -> Self;
    fn from_scale_translation<V: Ve<f32, D>>(scale: V, translation: V) -> Self;

    fn inverse(&self) -> Self;
    fn mul_vector<V: Ve<f32, D>>(&self, v: V) -> V;
}

impl Ma<3> for Mat4 {
    fn index(&self, i: usize, j: usize) -> f32 {
        self.col(i)[j]
    }

    fn from_scale<V: Ve<f32, 3>>(scale: V) -> Self {
        Mat4::from_scale_rotation_translation(scale.ve_into(), Quat::IDENTITY, Vec3::ZERO)
    }

    fn from_scale_translation<V: Ve<f32, 3>>(scale: V, translation: V) -> Self {
        Mat4::from_scale_rotation_translation(scale.ve_into(), Quat::IDENTITY, translation.ve_into())
    }

    fn inverse(&self) -> Self { Mat4::inverse(&self) }

    fn mul_vector<V: Ve<f32, 3>>(&self, v: V) -> V { 
        let v: Vec3 = v.ve_into();
        V::ve_from(self.mul_vec4(Vec4::from((v, 1.0)))) 
    }
}

impl CastFrom<Mat3> for Mat3 { fn cast_from(t: Mat3) -> Self { t } }
impl CastFrom<Mat4> for Mat3 { fn cast_from(t: Mat4) -> Self { unreachable!() } }

impl CastInto<Mat3> for Mat3 { fn cast_into(self) -> Mat3 { self } }
impl CastInto<Mat4> for Mat3 { fn cast_into(self) -> Mat4 { unreachable!() } }

impl Ma<2> for Mat3 {
    fn index(&self, i: usize, j: usize) -> f32 {
        self.col(i)[j]
    }

    fn from_scale<V: Ve<f32, 2>>(scale: V) -> Self {
        Mat3::from_scale_angle_translation(scale.ve_into(), 0.0, Vec2::ZERO)
    }

    fn from_scale_translation<V: Ve<f32, 2>>(scale: V, translation: V) -> Self {
        Mat3::from_scale_angle_translation(scale.ve_into(), 0.0, translation.ve_into())
    }

    fn inverse(&self) -> Self { Mat3::inverse(&self) }

    fn mul_vector<V: Ve<f32, 2>>(&self, v: V) -> V { V::ve_from(self.mul_vec3a(Vec3A::from((v.ve_into(), 1.0)))) }
}

impl CastFrom<Mat3> for Mat4 { fn cast_from(t: Mat3) -> Self { unreachable!() } }
impl CastFrom<Mat4> for Mat4 { fn cast_from(t: Mat4) -> Self { t } }

impl CastInto<Mat3> for Mat4 { fn cast_into(self) -> Mat3 { unreachable!() } }
impl CastInto<Mat4> for Mat4 { fn cast_into(self) -> Mat4 { self } }

