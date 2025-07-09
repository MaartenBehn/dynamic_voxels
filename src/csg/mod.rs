use std::fmt::Debug;

use crate::voxel::renderer::palette::MATERIAL_ID_BASE;

pub mod fast_query_csg_tree;
pub mod slot_map_csg_tree;
pub mod vec_csg_tree;

pub trait Base {
    fn base() -> Self;
}

impl Base for () {
    fn base() -> Self { () }
}

impl Base for u8 {
    fn base() -> Self { MATERIAL_ID_BASE }
}
