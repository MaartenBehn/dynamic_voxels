use std::fmt::Debug;
use crate::voxel::palette::palette::MATERIAL_ID_BASE;

pub mod csg_tree;
pub mod union;
pub mod sphere;
pub mod r#box;
pub mod all;

pub trait Base: Copy {
    fn base() -> Self;
}

impl Base for () {
    fn base() -> Self { () }
}

impl Base for u8 {
    fn base() -> Self { MATERIAL_ID_BASE }
}
