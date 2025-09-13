use core::fmt;
use std::{ops::{Add, AddAssign, Deref, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}};

use bvh::bounding_hierarchy::BHValue;

pub trait Nu: 
    Copy 
    + Default
    + fmt::Debug
    + Send
    + Sync
    + Sub<Output = Self> 
    + Add<Output = Self> 
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + PartialEq
    + PartialOrd 
    + 'static
{
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const EPSILON: Self;
    const MIN: Self;
    const MAX: Self;

    fn from_usize(v: usize) -> Self;
    fn from_f32(v: f32) -> Self;
    fn from_i32(v: i32) -> Self;
    fn to_usize(self) -> usize;
    fn to_f32(self) -> f32;
}

impl Nu for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const EPSILON: Self = f32::EPSILON;
    const MAX: Self = f32::MAX;
    const MIN: Self = f32::MIN;

    fn from_usize(v: usize) -> Self { v as f32 }
    fn from_f32(v: f32) -> Self { v }
    fn from_i32(v: i32) -> Self { v as f32 }
    fn to_usize(self) -> usize { self as usize } 
    fn to_f32(self) -> f32 { self } 
}

impl Nu for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const EPSILON: Self = 0;
    const MAX: Self = i32::MAX;
    const MIN: Self = i32::MIN;

    fn from_usize(v: usize) -> Self { v as i32 }
    fn from_f32(v: f32) -> Self { v as i32 }
    fn from_i32(v: i32) -> Self { v }
    fn to_usize(self) -> usize { self as usize } 
    fn to_f32(self) -> f32 { self as f32 } 
}
impl Nu for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const EPSILON: Self = 0;
    const MAX: Self = u32::MAX;
    const MIN: Self = u32::MIN;

    fn from_usize(v: usize) -> Self { v as u32 }
    fn from_f32(v: f32) -> Self { v as u32 }
    fn from_i32(v: i32) -> Self { v as u32 }
    fn to_usize(self) -> usize { self as usize } 
    fn to_f32(self) -> f32 { self as f32 } 
}
