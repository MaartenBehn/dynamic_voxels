use std::{cmp::Ordering, fmt::Debug, iter, ops::{Add, Div, Index, Mul, Neg, Sub}, process::Output};

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
    //+ Neg<Output = Self>
    + Index<usize, Output = T>
    + PartialEq
    + MC<Self, T, D>
    + CastFrom<Vec2>
    + CastFrom<IVec2>
    + CastFrom<Vec3>
    + CastFrom<Vec3A>
    + CastFrom<IVec3>
    + CastFrom<UVec3>
    + CastInto<Vec2>
    + CastInto<IVec2>
    + CastInto<Vec3>
    + CastInto<Vec3A>
    + CastInto<IVec3>
    + CastInto<UVec3>
    + FromT<Vec2>
    + FromT<IVec2>
    + FromT<Vec3>
    + FromT<Vec3A>
    + FromT<IVec3>
    + FromT<UVec3>
    + FromT<Vec4>
    + IntoT<Vec2>
    + IntoT<IVec2>
    + IntoT<Vec3>
    + IntoT<Vec3A>
    + IntoT<IVec3>
    + IntoT<UVec3>
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
    fn signum(self) -> Self;
    fn abs(self) -> Self;

    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn lt(self, other: Self) -> Self;
    fn gt(self, other: Self) -> Self;

    fn cmp(self, other: Self) -> Ordering;

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
    
    fn to_array(&self) -> [T; D];

    fn max_value(self) -> (usize, T) {

        let mut e_max = T::ZERO;
        let mut i_max = 0;
        for i in 0..D {
            let e = self.index(i);
            if *e > e_max {
                e_max = *e;
                i_max = i;
            }
        }

        (i_max, e_max)
    } 
}

pub trait CastFrom<T> {
    fn cast_from(t: T) -> Self;
}

pub trait CastInto<T> {
    fn cast_into(self) -> T;
}

pub trait FromT<T> {
    fn ve_from(t: T) -> Self;
}

pub trait IntoT<T> {
    fn ve_into(self) -> T;
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
    fn signum(self) -> Vec2 { Vec2::signum(self) }
    fn abs(self) -> Vec2 { Vec2::abs(self) }

    fn min(self, other: Self) -> Self { Vec2::min(self, other) }
    fn max(self, other: Self) -> Self { Vec2::max(self, other) }
    fn lt(self, other: Self) -> Self { Vec2::from(Vec2::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { Vec2::from(Vec2::cmpgt(self, other)) }

    fn cmp(self, other: Self) -> Ordering { self.x.total_cmp(&other.x).then(self.y.total_cmp(&other.y)) }

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

    fn to_vecf<V: Ve<f32, 2>>(self) -> V { V::ve_from(self) }
    fn from_vecf<V: Ve<f32, 2>>(v: V) -> Self { v.ve_into() }
    fn to_array(&self) -> [f32; 2] { Self::to_array(self) }
}

impl MC<Vec2, f32, 2> for Vec2 {
    type Matrix = Mat3;
    type VectorF = Vec2;

    fn to_vector(v: Self::VectorF) -> Self { v }
    fn to_vector_f(v: Self) -> Self::VectorF { v }
}

impl CastFrom<Vec2> for Vec2 { fn cast_from(t: Vec2) -> Self { t } }
impl CastFrom<IVec2> for Vec2 { fn cast_from(t: IVec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3> for Vec2 { fn cast_from(t: Vec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3A> for Vec2 { fn cast_from(t: Vec3A) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec3> for Vec2 { fn cast_from(t: IVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<UVec3> for Vec2 { fn cast_from(t: UVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }

impl CastInto<Vec2> for Vec2 { fn cast_into(self) -> Vec2 { self } }
impl CastInto<IVec2> for Vec2 { fn cast_into(self) -> IVec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3> for Vec2 { fn cast_into(self) -> Vec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3A> for Vec2 { fn cast_into(self) -> Vec3A { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec3> for Vec2 { fn cast_into(self) -> IVec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<UVec3> for Vec2 { fn cast_into(self) -> UVec3 { unsafe { cast_failed(); } unreachable!() } }

impl FromT<Vec2> for Vec2 { fn ve_from(t: Vec2) -> Self { t } } 
impl FromT<IVec2> for Vec2 { fn ve_from(t: IVec2) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<Vec3> for Vec2 { fn ve_from(t: Vec3) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<Vec3A> for Vec2 { fn ve_from(t: Vec3A) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<IVec3> for Vec2 { fn ve_from(t: IVec3) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<UVec3> for Vec2 { fn ve_from(t: UVec3) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<Vec4> for Vec2 { fn ve_from(t: Vec4) -> Self { Self::new(t.x as _, t.y as _) } }

impl IntoT<Vec2> for Vec2 { fn ve_into(self) -> Vec2 { self }  }
impl IntoT<IVec2> for Vec2 { fn ve_into(self) -> IVec2 { IVec2::new(self.x as _, self.y as _) }  }
impl IntoT<Vec3> for Vec2 { fn ve_into(self) -> Vec3 { Vec3::new(self.x as _, self.y as _, 0.0) } }
impl IntoT<Vec3A> for Vec2 { fn ve_into(self) -> Vec3A { Vec3A::new(self.x as _, self.y as _, 0.0) } }
impl IntoT<IVec3> for Vec2 { fn ve_into(self) -> IVec3 { IVec3::new(self.x as _, self.y as _, 0) } }
impl IntoT<UVec3> for Vec2 { fn ve_into(self) -> UVec3 { UVec3::new(self.x as _, self.y as _, 0) } }

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
    fn signum(self) -> Vec3 { Vec3::signum(self) }
    fn abs(self) -> Vec3 { Vec3::abs(self) }

    fn min(self, other: Self) -> Self { Vec3::min(self, other) }
    fn max(self, other: Self) -> Self { Vec3::max(self, other) }
    fn lt(self, other: Self) -> Self { Vec3::from(Vec3::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { Vec3::from(Vec3::cmpgt(self, other)) }
    
    fn cmp(self, other: Self) -> Ordering { self.x.total_cmp(&other.x).then(self.y.total_cmp(&other.y)).then(self.z.total_cmp(&other.z)) }

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

    fn to_vecf<V: Ve<f32, 3>>(self) -> V { V::ve_from(self) }
    fn from_vecf<V: Ve<f32, 3>>(v: V) -> Self { v.ve_into() }
    fn to_array(&self) -> [f32; 3] { Self::to_array(self) }
}

impl MC<Vec3, f32, 3> for Vec3 {
    type Matrix = Mat4;
    type VectorF = Vec3;

    fn to_vector(v: Self::VectorF) -> Self { v }
    fn to_vector_f(v: Self) -> Self::VectorF { v }
}

impl CastFrom<Vec2> for Vec3 { fn cast_from(t: Vec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec2> for Vec3 { fn cast_from(t: IVec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3> for Vec3 { fn cast_from(t: Vec3) -> Self { t } }
impl CastFrom<Vec3A> for Vec3 { fn cast_from(t: Vec3A) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec3> for Vec3 { fn cast_from(t: IVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<UVec3> for Vec3 { fn cast_from(t: UVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }

impl CastInto<Vec2> for Vec3 { fn cast_into(self) -> Vec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec2> for Vec3 { fn cast_into(self) -> IVec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3> for Vec3 { fn cast_into(self) -> Vec3 { self } }
impl CastInto<Vec3A> for Vec3 { fn cast_into(self) -> Vec3A { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec3> for Vec3 { fn cast_into(self) -> IVec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<UVec3> for Vec3 { fn cast_into(self) -> UVec3 { unsafe { cast_failed(); } unreachable!() } }

impl FromT<Vec2> for Vec3 { fn ve_from(t: Vec2) -> Self { Self::new(t.x as _, t.y as _, 0.0) } } 
impl FromT<IVec2> for Vec3 { fn ve_from(t: IVec2) -> Self { Self::new(t.x as _, t.y as _, 0.0) } }
impl FromT<Vec3> for Vec3 { fn ve_from(t: Vec3) -> Self { t } }
impl FromT<Vec3A> for Vec3 { fn ve_from(t: Vec3A) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<IVec3> for Vec3 { fn ve_from(t: IVec3) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<UVec3> for Vec3 { fn ve_from(t: UVec3) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<Vec4> for Vec3 { fn ve_from(t: Vec4) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }

impl IntoT<Vec2> for Vec3 { fn ve_into(self) -> Vec2 { Vec2::new(self.x as _, self.y as _) }  }
impl IntoT<IVec2> for Vec3 { fn ve_into(self) -> IVec2 { IVec2::new(self.x as _, self.y as _) }  }
impl IntoT<Vec3> for Vec3 { fn ve_into(self) -> Vec3 { self } }
impl IntoT<Vec3A> for Vec3 { fn ve_into(self) -> Vec3A { Vec3A::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<IVec3> for Vec3 { fn ve_into(self) -> IVec3 { IVec3::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<UVec3> for Vec3 { fn ve_into(self) -> UVec3 { UVec3::new(self.x as _, self.y as _, self.z as _) } }

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
    fn signum(self) -> Vec3A { Vec3A::signum(self) }
    fn abs(self) -> Vec3A { Vec3A::abs(self) }

    fn min(self, other: Self) -> Self { Vec3A::min(self, other) }
    fn max(self, other: Self) -> Self { Vec3A::max(self, other) }
    fn lt(self, other: Self) -> Self { Vec3A::from(Vec3A::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { Vec3A::from(Vec3A::cmpgt(self, other)) }
    
    fn cmp(self, other: Self) -> Ordering { self.x.total_cmp(&other.x).then(self.y.total_cmp(&other.y)).then(self.z.total_cmp(&other.z)) }

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

    fn to_vecf<V: Ve<f32, 3>>(self) -> V { V::ve_from(self) }
    fn from_vecf<V: Ve<f32, 3>>(v: V) -> Self { v.ve_into() }
    fn to_array(&self) -> [f32; 3] { Self::to_array(self) } 
}

impl MC<Vec3A, f32, 3> for Vec3A {
    type Matrix = Mat4;
    type VectorF = Vec3A;

    fn to_vector(v: Self::VectorF) -> Self { v }
    fn to_vector_f(v: Self) -> Self::VectorF { v }
}

impl CastFrom<Vec2> for Vec3A { fn cast_from(t: Vec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec2> for Vec3A { fn cast_from(t: IVec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3> for Vec3A { fn cast_from(t: Vec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3A> for Vec3A { fn cast_from(t: Vec3A) -> Self { t } }
impl CastFrom<IVec3> for Vec3A { fn cast_from(t: IVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<UVec3> for Vec3A { fn cast_from(t: UVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }

impl CastInto<Vec2> for Vec3A { fn cast_into(self) -> Vec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec2> for Vec3A { fn cast_into(self) -> IVec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3> for Vec3A { fn cast_into(self) -> Vec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3A> for Vec3A { fn cast_into(self) -> Vec3A { self } }
impl CastInto<IVec3> for Vec3A { fn cast_into(self) -> IVec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<UVec3> for Vec3A { fn cast_into(self) -> UVec3 { unsafe { cast_failed(); } unreachable!() } }

impl FromT<Vec2> for Vec3A { fn ve_from(t: Vec2) -> Self { Self::new(t.x, t.y, 0.0) } } 
impl FromT<IVec2> for Vec3A { fn ve_from(t: IVec2) -> Self { Self::new(t.x as _, t.y as _, 0.0) } }
impl FromT<Vec3> for Vec3A { fn ve_from(t: Vec3) -> Self { Self::new(t.x, t.y, t.z) } }
impl FromT<Vec3A> for Vec3A { fn ve_from(t: Vec3A) -> Self { t } }
impl FromT<IVec3> for Vec3A { fn ve_from(t: IVec3) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<UVec3> for Vec3A { fn ve_from(t: UVec3) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<Vec4> for Vec3A { fn ve_from(t: Vec4) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }


impl IntoT<Vec2> for Vec3A { fn ve_into(self) -> Vec2 { Vec2::new(self.x as _, self.y as _) }  }
impl IntoT<IVec2> for Vec3A { fn ve_into(self) -> IVec2 { IVec2::new(self.x as _, self.y as _) }  }
impl IntoT<Vec3> for Vec3A { fn ve_into(self) -> Vec3 { Vec3::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<Vec3A> for Vec3A { fn ve_into(self) -> Vec3A { self } }
impl IntoT<IVec3> for Vec3A { fn ve_into(self) -> IVec3 { IVec3::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<UVec3> for Vec3A { fn ve_into(self) -> UVec3 { UVec3::new(self.x as _, self.y as _, self.z as _) } }


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
    fn signum(self) -> IVec3 { IVec3::signum(self) }
    fn abs(self) -> IVec3 { IVec3::abs(self) }

    fn min(self, other: Self) -> Self { IVec3::min(self, other) }
    fn max(self, other: Self) -> Self { IVec3::max(self, other) }

    fn lt(self, other: Self) -> Self { IVec3::from(IVec3::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { IVec3::from(IVec3::cmpgt(self, other)) }
    
    fn cmp(self, other: Self) -> Ordering { self.x.cmp(&other.x).then(self.y.cmp(&other.y)).then(self.z.cmp(&other.z)) }

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

    fn to_vecf<V: Ve<f32, 3>>(self) -> V { V::ve_from(self) }
    fn from_vecf<V: Ve<f32, 3>>(v: V) -> Self { v.ve_into() }
    fn to_array(&self) -> [i32; 3] { Self::to_array(self) }
}

impl MC<IVec3, i32, 3> for IVec3 {
    type Matrix = Mat4;
    type VectorF = Vec3A;

    fn to_vector(v: Self::VectorF) -> Self { v.as_ivec3() }
    fn to_vector_f(v: Self) -> Self::VectorF { v.as_vec3a() }
}

impl CastFrom<Vec2> for IVec3 { fn cast_from(t: Vec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec2> for IVec3 { fn cast_from(t: IVec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3> for IVec3 { fn cast_from(t: Vec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3A> for IVec3 { fn cast_from(t: Vec3A) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec3> for IVec3 { fn cast_from(t: IVec3) -> Self { t } }
impl CastFrom<UVec3> for IVec3 { fn cast_from(t: UVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }

impl CastInto<Vec2> for IVec3 { fn cast_into(self) -> Vec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec2> for IVec3 { fn cast_into(self) -> IVec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3> for IVec3 { fn cast_into(self) -> Vec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3A> for IVec3 { fn cast_into(self) -> Vec3A { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec3> for IVec3 { fn cast_into(self) -> IVec3 { self } }
impl CastInto<UVec3> for IVec3 { fn cast_into(self) -> UVec3 { unsafe { cast_failed(); } unreachable!() } }

impl FromT<Vec2> for IVec3 { fn ve_from(t: Vec2) -> Self { Self::new(t.x as _, t.y as _, 0) } } 
impl FromT<IVec2> for IVec3 { fn ve_from(t: IVec2) -> Self { Self::new(t.x as _, t.y as _, 0) } }
impl FromT<Vec3> for IVec3 { fn ve_from(t: Vec3) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<Vec3A> for IVec3 { fn ve_from(t: Vec3A) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<IVec3> for IVec3 { fn ve_from(t: IVec3) -> Self { t } }
impl FromT<UVec3> for IVec3 { fn ve_from(t: UVec3) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<Vec4> for IVec3 { fn ve_from(t: Vec4) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }


impl IntoT<Vec2> for IVec3 { fn ve_into(self) -> Vec2 { Vec2::new(self.x as _, self.y as _) }  }
impl IntoT<IVec2> for IVec3 { fn ve_into(self) -> IVec2 { IVec2::new(self.x as _, self.y as _) }  }
impl IntoT<Vec3> for IVec3 { fn ve_into(self) -> Vec3 { Vec3::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<Vec3A> for IVec3 { fn ve_into(self) -> Vec3A { Vec3A::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<IVec3> for IVec3 { fn ve_into(self) -> IVec3 { self } }
impl IntoT<UVec3> for IVec3 { fn ve_into(self) -> UVec3 { UVec3::new(self.x as _, self.y as _, self.z as _) } }



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
    fn signum(self) -> IVec2 { IVec2::signum(self) }
    fn abs(self) -> IVec2 { IVec2::abs(self) }
    
    fn min(self, other: Self) -> Self { IVec2::min(self, other) }
    fn max(self, other: Self) -> Self { IVec2::max(self, other) }

    fn lt(self, other: Self) -> Self { IVec2::from(IVec2::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { IVec2::from(IVec2::cmpgt(self, other)) }
    
    fn cmp(self, other: Self) -> Ordering { self.x.cmp(&other.x).then(self.y.cmp(&other.y)) }

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

    fn to_vecf<V: Ve<f32, 2>>(self) -> V { V::ve_from(self) }
    fn from_vecf<V: Ve<f32, 2>>(v: V) -> Self { v.ve_into() }
    fn to_array(&self) -> [i32; 2] { Self::to_array(self) }
}

impl MC<IVec2, i32, 2> for IVec2 {
    type Matrix = Mat3;
    type VectorF = Vec2;

    fn to_vector(v: Self::VectorF) -> Self { v.as_ivec2() }
    fn to_vector_f(v: Self) -> Self::VectorF { v.as_vec2() }
}

impl CastFrom<Vec2> for IVec2 { fn cast_from(t: Vec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec2> for IVec2 { fn cast_from(t: IVec2) -> Self { t } }
impl CastFrom<Vec3> for IVec2 { fn cast_from(t: Vec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3A> for IVec2 { fn cast_from(t: Vec3A) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec3> for IVec2 { fn cast_from(t: IVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<UVec3> for IVec2 { fn cast_from(t: UVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }

impl CastInto<Vec2> for IVec2 { fn cast_into(self) -> Vec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec2> for IVec2 { fn cast_into(self) -> IVec2 { self } }
impl CastInto<Vec3> for IVec2 { fn cast_into(self) -> Vec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3A> for IVec2 { fn cast_into(self) -> Vec3A { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec3> for IVec2 { fn cast_into(self) -> IVec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<UVec3> for IVec2 { fn cast_into(self) -> UVec3 { unsafe { cast_failed(); } unreachable!() } }

impl FromT<Vec2> for IVec2 { fn ve_from(t: Vec2) -> Self { Self::new(t.x as _, t.y as _) } } 
impl FromT<IVec2> for IVec2 { fn ve_from(t: IVec2) -> Self { t } }
impl FromT<Vec3> for IVec2 { fn ve_from(t: Vec3) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<Vec3A> for IVec2 { fn ve_from(t: Vec3A) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<IVec3> for IVec2 { fn ve_from(t: IVec3) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<UVec3> for IVec2 { fn ve_from(t: UVec3) -> Self { Self::new(t.x as _, t.y as _) } }
impl FromT<Vec4> for IVec2 { fn ve_from(t: Vec4) -> Self { Self::new(t.x as _, t.y as _) } }


impl IntoT<Vec2> for IVec2 { fn ve_into(self) -> Vec2 { Vec2::new(self.x as _, self.y as _) }  }
impl IntoT<IVec2> for IVec2 { fn ve_into(self) -> IVec2 { self }  }
impl IntoT<Vec3> for IVec2 { fn ve_into(self) -> Vec3 { Vec3::new(self.x as _, self.y as _, 0.0) } }
impl IntoT<Vec3A> for IVec2 { fn ve_into(self) -> Vec3A { Vec3A::new(self.x as _, self.y as _, 0.0) } }
impl IntoT<IVec3> for IVec2 { fn ve_into(self) -> IVec3 { IVec3::new(self.x as _, self.y as _, 0) } }
impl IntoT<UVec3> for IVec2 { fn ve_into(self) -> UVec3 { UVec3::new(self.x as _, self.y as _, 0) } }



impl Ve<u32, 3> for UVec3 {
    const ZERO: Self = UVec3::ZERO;
    const ONE: Self = UVec3::ONE;
    const MIN: Self = UVec3::MIN;
    const MAX: Self = UVec3::MAX;

    fn new(v: [u32; 3]) -> Self { UVec3::from_array(v) }
    fn from_iter<I: Iterator<Item = u32>>(iter: &mut I) -> Self {
        let mut v = UVec3::ZERO;
        v.x = iter.next().unwrap();
        v.y = iter.next().unwrap();
        v.z = iter.next().unwrap();
        v
    }

    fn dot(self, other: Self) -> u32 { UVec3::dot(self, other) }
    fn length_squared(self) -> u32 { UVec3::length_squared(self) }
    fn length(self) -> u32 { self.as_vec3a().length() as u32 }
    fn normalize(self) -> UVec3 { self.as_vec3a().normalize().as_uvec3() }
    fn element_sum(self) -> u32 { UVec3::element_sum(self) }
    fn signum(self) -> UVec3 { unreachable!() }
    fn abs(self) -> UVec3 { self }

    fn min(self, other: Self) -> Self { UVec3::min(self, other) }
    fn max(self, other: Self) -> Self { UVec3::max(self, other) }

    fn lt(self, other: Self) -> Self { UVec3::from(UVec3::cmplt(self, other)) }
    fn gt(self, other: Self) -> Self { UVec3::from(UVec3::cmpgt(self, other)) }
    
    fn cmp(self, other: Self) -> Ordering { self.x.cmp(&other.x).then(self.y.cmp(&other.y)).then(self.z.cmp(&other.z)) }

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

    fn to_vecf<V: Ve<f32, 3>>(self) -> V { V::ve_from(self) }
    fn from_vecf<V: Ve<f32, 3>>(v: V) -> Self { v.ve_into() }
    fn to_array(&self) -> [u32; 3] { Self::to_array(self) }
}

impl MC<UVec3, u32, 3> for UVec3 {
    type Matrix = Mat4;
    type VectorF = Vec3A;

    fn to_vector(v: Self::VectorF) -> Self { v.as_uvec3() }
    fn to_vector_f(v: Self) -> Self::VectorF { v.as_vec3a() }
}

impl CastFrom<Vec2> for UVec3 { fn cast_from(t: Vec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec2> for UVec3 { fn cast_from(t: IVec2) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3> for UVec3 { fn cast_from(t: Vec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<Vec3A> for UVec3 { fn cast_from(t: Vec3A) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<IVec3> for UVec3 { fn cast_from(t: IVec3) -> Self { unsafe { cast_failed(); } unreachable!() } }
impl CastFrom<UVec3> for UVec3 { fn cast_from(t: UVec3) -> Self { t } }

impl CastInto<Vec2> for UVec3 { fn cast_into(self) -> Vec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec2> for UVec3 { fn cast_into(self) -> IVec2 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3> for UVec3 { fn cast_into(self) -> Vec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<Vec3A> for UVec3 { fn cast_into(self) -> Vec3A { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<IVec3> for UVec3 { fn cast_into(self) -> IVec3 { unsafe { cast_failed(); } unreachable!() } }
impl CastInto<UVec3> for UVec3 { fn cast_into(self) -> UVec3 { self } }

impl FromT<Vec2> for UVec3 { fn ve_from(t: Vec2) -> Self { Self::new(t.x as _, t.y as _, 0) } } 
impl FromT<IVec2> for UVec3 { fn ve_from(t: IVec2) -> Self { Self::new(t.x as _, t.y as _, 0) } }
impl FromT<Vec3> for UVec3 { fn ve_from(t: Vec3) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<Vec3A> for UVec3 { fn ve_from(t: Vec3A) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<IVec3> for UVec3 { fn ve_from(t: IVec3) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }
impl FromT<UVec3> for UVec3 { fn ve_from(t: UVec3) -> Self { t } }
impl FromT<Vec4> for UVec3 { fn ve_from(t: Vec4) -> Self { Self::new(t.x as _, t.y as _, t.z as _) } }


impl IntoT<Vec2> for UVec3 { fn ve_into(self) -> Vec2 { Vec2::new(self.x as _, self.y as _) }  }
impl IntoT<IVec2> for UVec3 { fn ve_into(self) -> IVec2 { IVec2::new(self.x as _, self.y as _) }  }
impl IntoT<Vec3> for UVec3 { fn ve_into(self) -> Vec3 { Vec3::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<Vec3A> for UVec3 { fn ve_into(self) -> Vec3A { Vec3A::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<IVec3> for UVec3 { fn ve_into(self) -> IVec3 { IVec3::new(self.x as _, self.y as _, self.z as _) } }
impl IntoT<UVec3> for UVec3 { fn ve_into(self) -> UVec3 { self } }


fn cast_failed() {}

/*
unsafe extern "C" {
    #[link_name = "\n\nERROR: Invalid cast!\n"]
    fn cast_failed();
}
*/
