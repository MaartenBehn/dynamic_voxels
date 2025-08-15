use bvh::bounding_hierarchy::BHValue;

pub trait Nu: Copy + PartialEq + PartialOrd {
    const TWO: Self;
}

impl Nu for f32 {
    const TWO: Self = 2.0;
}
impl Nu for i32 {
    const TWO: Self = 2;
}
impl Nu for u32 {
    const TWO: Self = 2;
}
