use std::{fmt::Debug, iter, ops::{Add, Div, Index, Mul, Neg, Sub}, process::Output};

use octa_force::glam::{vec2, IVec2, IVec3, Mat3, Mat4, UVec3, Vec2, Vec3, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles};

use super::{math, math_config::MC, number::Nu};

pub trait Ve<T: Nu, const D: usize>: 
    Sized    
    + Copy
    + Debug
    + Default
    + Send
    + Sync
    + Sub<Output = Self> 
    + Sub<T, Output = Self> 
    + Add<Output = Self> 
    + Add<T, Output = Self> 
    + Mul<Output = Self>
    + Mul<T, Output = Self>
    + Div<Output = Self>
    + Div<T, Output = Self>
    + Neg<Output = Self>
    + Index<usize, Output = T>
    + PartialEq
    + MC<Self, T, D>
    + 'static
{
    const ZERO: Self;
    const ONE: Self;
    const MIN: Self;
    const MAX: Self;

    fn new(v: [T; D]) -> Self;
    fn from_iter<I: Iterator<Item = T>>(iter: &mut I) -> Self;
    
    fn dot(self, other: Self) -> T;
    fn length_squared(self) -> T;
    fn length(self) -> T;
    fn normalize(self) -> Self;
    fn element_sum(self) -> T;

    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn lt(self, other: Self) -> Self;
    fn gt(self, other: Self) -> Self;

    fn cmplt_any(self, other: Self) -> bool;
    fn cmpgt_any(self, other: Self) -> bool;
    fn cmple_any(self, other: Self) -> bool;
    fn cmpge_any(self, other: Self) -> bool;
    fn cmpeq_any(self, other: Self) -> bool;
    fn cmpne_any(self, other: Self) -> bool;
    fn cmplt_all(self, other: Self) -> bool;
    fn cmpgt_all(self, other: Self) -> bool;
    fn cmple_all(self, other: Self) -> bool;
    fn cmpge_all(self, other: Self) -> bool;
    fn cmpeq_all(self, other: Self) -> bool;
    fn cmpne_all(self, other: Self) -> bool;

    fn to_vecf<V: Ve<f32, D>>(self) -> V;
    fn from_vecf<V: Ve<f32, D>>(v: V) -> Self;
    
    fn to_vec3a(self) -> Vec3A;
    fn to_vec3(self) -> Vec3;
    fn to_vec2(self) -> Vec2;
    fn to_ivec3(self) -> IVec3;
    fn to_ivec2(self) -> IVec2;
    fn to_uvec3(self) -> UVec3;
   
    fn from_vec4h(v: Vec4) -> Self;
    fn from_vec3a(v: Vec3A) -> Self;
    fn from_vec3(v: Vec3) -> Self;
    fn from_vec2(v: Vec2) -> Self;
    fn from_ivec3(v: IVec3) -> Self;
    fn from_ivec2(v: IVec2) -> Self;
    fn from_uvec3(v: UVec3) -> Self;

    fn to_array(&self) -> [T; D];

    fn max_index(self) -> usize {

        let mut e_max = T::ZERO;
        let mut i_max = 0;
        for i in 0..D {
            let e = self.index(i);
            if *e > e_max {
                e_max = *e;
                i_max = i;
            }
        }

        i_max
    }

}

impl Ve<f32, 2> for Vec2 {
    const ZERO: Self = Vec2::ZERO;
    const ONE: Self = Vec2::ONE;
    const MIN: Self = Vec2::MIN;
    const MAX: Self = Vec2::MAX;

    fn new(v: [f32; 2]) -> Self { Vec2::from_array(v) }
    fn from_iter<I: Iterator<Item = f32>>(iter: &mut I) -> Self {
        let mut v = Vec2::ZERO;
        v.x = iter.next().unwrap();
        v.y = iter.next().unwrap();
        v
    }

    fn dot(self, other: Self) -> f32 { Vec2::dot(self, other) }
    fn length_squared(self) -> f32 { Vec2::length_squared(self) }
    fn length(self) -> f32 { Vec2::length(self) }
    fn normalize(self) -> Vec2 { Vec2::normalize(self) }
    fn element_sum(self) -> f32 { Vec2::element_sum(self) }

    fn min(self, other: Self) -> Self { Vec2::min(self, other) }
    fn max(self, other: Self) -> Self { Vec2::max(self, other) }
    fn lt(self, other: Self) -> Self { Vec2::from(Vec2::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { Vec2::from(Vec2::cmpgt(self, other)) }
    
    fn cmplt_any(self, other: Self) -> bool { self.cmplt(other).any() }
    fn cmpgt_any(self, other: Self) -> bool { self.cmpgt(other).any() }
    fn cmple_any(self, other: Self) -> bool { self.cmple(other).any() }
    fn cmpge_any(self, other: Self) -> bool { self.cmpge(other).any() }
    fn cmpeq_any(self, other: Self) -> bool { self.cmpeq(other).any() }
    fn cmpne_any(self, other: Self) -> bool { self.cmpne(other).any() }
    fn cmplt_all(self, other: Self) -> bool { self.cmplt(other).all() }
    fn cmpgt_all(self, other: Self) -> bool { self.cmpgt(other).all() }
    fn cmple_all(self, other: Self) -> bool { self.cmple(other).all() }
    fn cmpge_all(self, other: Self) -> bool { self.cmpge(other).all() }
    fn cmpeq_all(self, other: Self) -> bool { self.cmpeq(other).all() }
    fn cmpne_all(self, other: Self) -> bool { self.cmpne(other).all() }

    fn to_vecf<V: Ve<f32, 2>>(self) -> V { V::from_vec2(self) }
    fn from_vecf<V: Ve<f32, 2>>(v: V) -> Self { v.to_vec2() }

    fn to_vec3a(self) -> Vec3A { unreachable!() }
    fn to_vec3(self) -> Vec3 { unreachable!() }
    fn to_vec2(self) -> Vec2 { self }
    fn to_ivec3(self) -> IVec3 { unreachable!() }
    fn to_ivec2(self) -> IVec2 { self.as_ivec2() }
    fn to_uvec3(self) -> UVec3 { unreachable!() }

    fn from_vec4h(v: Vec4) -> Self { unreachable!() }
    fn from_vec3a(v: Vec3A) -> Self { v.xy() }
    fn from_vec3(v: Vec3) -> Self { unreachable!() }
    fn from_vec2(v:  Vec2) -> Self { v }
    fn from_ivec3(v: IVec3) -> Self { unreachable!() }
    fn from_ivec2(v: IVec2) -> Self { v.as_vec2() }
    fn from_uvec3(v: UVec3) -> Self { unreachable!() }

    fn to_array(&self) -> [f32; 2] { Self::to_array(self) }
}

impl MC<Vec2, f32, 2> for Vec2 {
    type Matrix = Mat3;
    type VectorF = Vec2;

    fn to_vector(v: Self::VectorF) -> Self { v }
    fn to_vector_f(v: Self) -> Self::VectorF { v }
}

impl Ve<f32, 3> for Vec3 {
    const ZERO: Self = Vec3::ZERO;
    const ONE: Self = Vec3::ONE;
    const MIN: Self = Vec3::MIN;
    const MAX: Self = Vec3::MAX;

    fn new(v: [f32; 3]) -> Self { Vec3::from_array(v) }
    fn from_iter<I: Iterator<Item = f32>>(iter: &mut I) -> Self {
        let mut v = Vec3::ZERO;
        v.x = iter.next().unwrap();
        v.y = iter.next().unwrap();
        v.z = iter.next().unwrap();
        v
    }
    
    fn dot(self, other: Self) -> f32 { Vec3::dot(self, other) }
    fn length_squared(self) -> f32 { Vec3::length_squared(self) }
    fn length(self) -> f32 { Vec3::length(self) }
    fn normalize(self) -> Vec3 { Vec3::normalize(self) }
    fn element_sum(self) -> f32 { Vec3::element_sum(self) }

    fn min(self, other: Self) -> Self { Vec3::min(self, other) }
    fn max(self, other: Self) -> Self { Vec3::max(self, other) }
    fn lt(self, other: Self) -> Self { Vec3::from(Vec3::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { Vec3::from(Vec3::cmpgt(self, other)) }

    fn cmplt_any(self, other: Self) -> bool { self.cmplt(other).any() }
    fn cmpgt_any(self, other: Self) -> bool { self.cmpgt(other).any() }
    fn cmple_any(self, other: Self) -> bool { self.cmple(other).any() }
    fn cmpge_any(self, other: Self) -> bool { self.cmpge(other).any() }
    fn cmpeq_any(self, other: Self) -> bool { self.cmpeq(other).any() }
    fn cmpne_any(self, other: Self) -> bool { self.cmpne(other).any() }
    fn cmplt_all(self, other: Self) -> bool { self.cmplt(other).all() }
    fn cmpgt_all(self, other: Self) -> bool { self.cmpgt(other).all() }
    fn cmple_all(self, other: Self) -> bool { self.cmple(other).all() }
    fn cmpge_all(self, other: Self) -> bool { self.cmpge(other).all() }
    fn cmpeq_all(self, other: Self) -> bool { self.cmpeq(other).all() }
    fn cmpne_all(self, other: Self) -> bool { self.cmpne(other).all() }

    fn to_vecf<V: Ve<f32, 3>>(self) -> V { V::from_vec3(self) }
    fn from_vecf<V: Ve<f32, 3>>(v: V) -> Self { v.to_vec3() }

    fn to_vec3a(self) -> Vec3A { Vec3A::from(self) }
    fn to_vec3(self) -> Vec3 { self }
    fn to_vec2(self) -> Vec2 { unreachable!() }
    fn to_ivec3(self) -> IVec3 { self.as_ivec3() }
    fn to_ivec2(self) -> IVec2 { unreachable!() }
    fn to_uvec3(self) -> UVec3 { self.as_uvec3() }

    fn from_vec4h(v: Vec4) -> Self { v.xyz() }
    fn from_vec3a(v: Vec3A) -> Self { Vec3::from(v) }
    fn from_vec3(v: Vec3) -> Self { v }
    fn from_vec2(v:  Vec2) -> Self { unreachable!() }
    fn from_ivec3(v: IVec3) -> Self { v.as_vec3() }
    fn from_ivec2(v: IVec2) -> Self { unreachable!() }
    fn from_uvec3(v: UVec3) -> Self { v.as_vec3() }

    fn to_array(&self) -> [f32; 3] { Self::to_array(self) }
}

impl MC<Vec3, f32, 3> for Vec3 {
    type Matrix = Mat4;
    type VectorF = Vec3;

    fn to_vector(v: Self::VectorF) -> Self { v }
    fn to_vector_f(v: Self) -> Self::VectorF { v }
}

impl Ve<f32, 3> for Vec3A {
    const ZERO: Self = Vec3A::ZERO;
    const ONE: Self = Vec3A::ONE;
    const MIN: Self = Vec3A::MIN;
    const MAX: Self = Vec3A::MAX;

    fn new(v: [f32; 3]) -> Self { Vec3A::from_array(v) }
    fn from_iter<I: Iterator<Item = f32>>(iter: &mut I) -> Self {
        let mut v = Vec3A::ZERO;
        v.x = iter.next().unwrap();
        v.y = iter.next().unwrap();
        v.z = iter.next().unwrap();
        v
    }
    
    fn dot(self, other: Self) -> f32 { Vec3A::dot(self, other) }
    fn length_squared(self) -> f32 { Vec3A::length_squared(self) }
    fn length(self) -> f32 { Vec3A::length(self) }
    fn normalize(self) -> Vec3A { Vec3A::normalize(self) }
    fn element_sum(self) -> f32 { Vec3A::element_sum(self) }

    fn min(self, other: Self) -> Self { Vec3A::min(self, other) }
    fn max(self, other: Self) -> Self { Vec3A::max(self, other) }
    fn lt(self, other: Self) -> Self { Vec3A::from(Vec3A::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { Vec3A::from(Vec3A::cmpgt(self, other)) }

    fn cmplt_any(self, other: Self) -> bool { self.cmplt(other).any() }
    fn cmpgt_any(self, other: Self) -> bool { self.cmpgt(other).any() }
    fn cmple_any(self, other: Self) -> bool { self.cmple(other).any() }
    fn cmpge_any(self, other: Self) -> bool { self.cmpge(other).any() }
    fn cmpeq_any(self, other: Self) -> bool { self.cmpeq(other).any() }
    fn cmpne_any(self, other: Self) -> bool { self.cmpne(other).any() }
    fn cmplt_all(self, other: Self) -> bool { self.cmplt(other).all() }
    fn cmpgt_all(self, other: Self) -> bool { self.cmpgt(other).all() }
    fn cmple_all(self, other: Self) -> bool { self.cmple(other).all() }
    fn cmpge_all(self, other: Self) -> bool { self.cmpge(other).all() }
    fn cmpeq_all(self, other: Self) -> bool { self.cmpeq(other).all() }
    fn cmpne_all(self, other: Self) -> bool { self.cmpne(other).all() }

    fn to_vecf<V: Ve<f32, 3>>(self) -> V { V::from_vec3a(self) }
    fn from_vecf<V: Ve<f32, 3>>(v: V) -> Self { v.to_vec3a() }

    fn to_vec3a(self) -> Vec3A { self }
    fn to_vec3(self) -> Vec3 { Vec3::from(self) }
    fn to_vec2(self) -> Vec2 { unreachable!() }
    fn to_ivec3(self) -> IVec3 { self.as_ivec3() }
    fn to_ivec2(self) -> IVec2 { unreachable!() }
    fn to_uvec3(self) -> UVec3 { self.as_uvec3() }

    fn from_vec4h(v: Vec4) -> Self { Vec3A::from(v) }
    fn from_vec3a(v: Vec3A) -> Self { v }
    fn from_vec3(v: Vec3) -> Self { Vec3A::from(v) }
    fn from_vec2(v:  Vec2) -> Self { unreachable!() }
    fn from_ivec3(v: IVec3) -> Self { v.as_vec3a() }
    fn from_ivec2(v: IVec2) -> Self { unreachable!() }
    fn from_uvec3(v: UVec3) -> Self { v.as_vec3a() }

    fn to_array(&self) -> [f32; 3] { Self::to_array(self) } 
}

impl MC<Vec3A, f32, 3> for Vec3A {
    type Matrix = Mat4;
    type VectorF = Vec3A;

    fn to_vector(v: Self::VectorF) -> Self { v }
    fn to_vector_f(v: Self) -> Self::VectorF { v }
}

impl Ve<i32, 3> for IVec3 {
    const ZERO: Self = IVec3::ZERO;
    const ONE: Self = IVec3::ONE;
    const MIN: Self = IVec3::MIN;
    const MAX: Self = IVec3::MAX;

    fn new(v: [i32; 3]) -> Self { IVec3::from_array(v) }
    fn from_iter<I: Iterator<Item = i32>>(iter: &mut I) -> Self {
        let mut v = IVec3::ZERO;
        v.x = iter.next().unwrap();
        v.y = iter.next().unwrap();
        v.z = iter.next().unwrap();
        v
    }

    fn dot(self, other: Self) -> i32 { IVec3::dot(self, other) }
    fn length_squared(self) -> i32 { IVec3::length_squared(self) }
    fn length(self) -> i32 { self.as_vec3a().length() as i32 }
    fn normalize(self) -> IVec3 { self.as_vec3a().normalize().as_ivec3() }
    fn element_sum(self) -> i32 { IVec3::element_sum(self) }

    fn min(self, other: Self) -> Self { IVec3::min(self, other) }
    fn max(self, other: Self) -> Self { IVec3::max(self, other) }

    fn lt(self, other: Self) -> Self { IVec3::from(IVec3::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { IVec3::from(IVec3::cmpgt(self, other)) }

    fn cmplt_any(self, other: Self) -> bool { self.cmplt(other).any() }
    fn cmpgt_any(self, other: Self) -> bool { self.cmpgt(other).any() }
    fn cmple_any(self, other: Self) -> bool { self.cmple(other).any() }
    fn cmpge_any(self, other: Self) -> bool { self.cmpge(other).any() }
    fn cmpeq_any(self, other: Self) -> bool { self.cmpeq(other).any() }
    fn cmpne_any(self, other: Self) -> bool { self.cmpne(other).any() }
    fn cmplt_all(self, other: Self) -> bool { self.cmplt(other).all() }
    fn cmpgt_all(self, other: Self) -> bool { self.cmpgt(other).all() }
    fn cmple_all(self, other: Self) -> bool { self.cmple(other).all() }
    fn cmpge_all(self, other: Self) -> bool { self.cmpge(other).all() }
    fn cmpeq_all(self, other: Self) -> bool { self.cmpeq(other).all() }
    fn cmpne_all(self, other: Self) -> bool { self.cmpne(other).all() }

    fn to_vecf<V: Ve<f32, 3>>(self) -> V { V::from_ivec3(self) }
    fn from_vecf<V: Ve<f32, 3>>(v: V) -> Self { v.to_ivec3() }

    fn to_vec3a(self) -> Vec3A { self.as_vec3a() }
    fn to_vec3(self) -> Vec3 { self.as_vec3() }
    fn to_vec2(self) -> Vec2 { unreachable!() }
    fn to_ivec3(self) -> IVec3 { self }
    fn to_ivec2(self) -> IVec2 { unreachable!() }
    fn to_uvec3(self) -> UVec3 { self.as_uvec3() }
    
    fn from_vec4h(v: Vec4) -> Self { v.xyz().as_ivec3() }
    fn from_vec3a(v: Vec3A) -> Self { v.as_ivec3() }
    fn from_vec3(v: Vec3) -> Self { v.as_ivec3() }
    fn from_vec2(v:  Vec2) -> Self { unreachable!() }
    fn from_ivec3(v: IVec3) -> Self { v }
    fn from_ivec2(v: IVec2) -> Self { unreachable!() }
    fn from_uvec3(v: UVec3) -> Self { v.as_ivec3() }

    fn to_array(&self) -> [i32; 3] { Self::to_array(self) }
}

impl MC<IVec3, i32, 3> for IVec3 {
    type Matrix = Mat4;
    type VectorF = Vec3A;

    fn to_vector(v: Self::VectorF) -> Self { v.as_ivec3() }
    fn to_vector_f(v: Self) -> Self::VectorF { v.as_vec3a() }
}

impl Ve<i32, 2> for IVec2 {
    const ZERO: Self = IVec2::ZERO;
    const ONE: Self = IVec2::ONE;
    const MIN: Self = IVec2::MIN;
    const MAX: Self = IVec2::MAX;

    fn new(v: [i32; 2]) -> Self { IVec2::from_array(v) }
    fn from_iter<I: Iterator<Item = i32>>(iter: &mut I) -> Self {
        let mut v = IVec2::ZERO;
        v.x = iter.next().unwrap();
        v.y = iter.next().unwrap();
        v
    }

    fn dot(self, other: Self) -> i32 { IVec2::dot(self, other) }
    fn length_squared(self) -> i32 { IVec2::length_squared(self) }
    fn length(self) -> i32 { self.as_vec2().length() as i32 }
    fn normalize(self) -> IVec2 { self.as_vec2().normalize().as_ivec2() }
    fn element_sum(self) -> i32 { IVec2::element_sum(self) }

    fn min(self, other: Self) -> Self { IVec2::min(self, other) }
    fn max(self, other: Self) -> Self { IVec2::max(self, other) }

    fn lt(self, other: Self) -> Self { IVec2::from(IVec2::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { IVec2::from(IVec2::cmpgt(self, other)) }

    fn cmplt_any(self, other: Self) -> bool { self.cmplt(other).any() }
    fn cmpgt_any(self, other: Self) -> bool { self.cmpgt(other).any() }
    fn cmple_any(self, other: Self) -> bool { self.cmple(other).any() }
    fn cmpge_any(self, other: Self) -> bool { self.cmpge(other).any() }
    fn cmpeq_any(self, other: Self) -> bool { self.cmpeq(other).any() }
    fn cmpne_any(self, other: Self) -> bool { self.cmpne(other).any() }
    fn cmplt_all(self, other: Self) -> bool { self.cmplt(other).all() }
    fn cmpgt_all(self, other: Self) -> bool { self.cmpgt(other).all() }
    fn cmple_all(self, other: Self) -> bool { self.cmple(other).all() }
    fn cmpge_all(self, other: Self) -> bool { self.cmpge(other).all() }
    fn cmpeq_all(self, other: Self) -> bool { self.cmpeq(other).all() }
    fn cmpne_all(self, other: Self) -> bool { self.cmpne(other).all() }

    fn to_vecf<V: Ve<f32, 2>>(self) -> V { V::from_ivec2(self) }
    fn from_vecf<V: Ve<f32, 2>>(v: V) -> Self { v.to_ivec2() }

    fn to_vec3a(self) -> Vec3A { unreachable!()  }
    fn to_vec3(self) -> Vec3 { unreachable!()  }
    fn to_vec2(self) -> Vec2 { self.as_vec2() }
    fn to_ivec3(self) -> IVec3 { unreachable!() }
    fn to_ivec2(self) -> IVec2 { self }
    fn to_uvec3(self) -> UVec3 { unreachable!() }
    
    fn from_vec4h(v: Vec4) -> Self { unreachable!() }
    fn from_vec3a(v: Vec3A) -> Self { v.xy().as_ivec2() }
    fn from_vec3(v: Vec3) -> Self { unreachable!() }
    fn from_vec2(v:  Vec2) -> Self { v.as_ivec2() }
    fn from_ivec3(v: IVec3) -> Self { unreachable!() }
    fn from_ivec2(v: IVec2) -> Self { v }
    fn from_uvec3(v: UVec3) -> Self { unreachable!() }

    fn to_array(&self) -> [i32; 2] { Self::to_array(self) }
}

impl MC<IVec2, i32, 2> for IVec2 {
    type Matrix = Mat3;
    type VectorF = Vec2;

    fn to_vector(v: Self::VectorF) -> Self { v.as_ivec2() }
    fn to_vector_f(v: Self) -> Self::VectorF { v.as_vec2() }
}

pub fn vector_to_nalgebra<V: Ve<f32, D>, const D: usize>(v: V) -> nalgebra::OPoint<f32, nalgebra::Const<D>> {
    nalgebra::OPoint::from_slice(&v.to_array())
}

pub fn nalgebra_to_vector<V: Ve<f32, D>, const D: usize>(v: nalgebra::OPoint<f32, nalgebra::Const<D>>) -> V {
    V::from_iter(&mut v.iter().copied())
}
