use std::fmt::Debug;

use crate::voxel::palette::palette::MATERIAL_ID_BASE;

pub mod csg_tree;
pub mod csg_tree_2d;
pub mod fast_query_csg_tree;

pub trait Base {
    fn base() -> Self;
}

impl Base for () {
    fn base() -> Self { () }
}

impl Base for u8 {
    fn base() -> Self { MATERIAL_ID_BASE }
}
