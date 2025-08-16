use core::fmt;

use bvh::bounding_hierarchy::BHValue;

pub trait Nu: 
    Copy 
    + Default
    + fmt::Debug
    + Send
    + Sync
    + PartialEq
    + PartialOrd 
{
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
}

impl Nu for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
}
impl Nu for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
}
impl Nu for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
}
