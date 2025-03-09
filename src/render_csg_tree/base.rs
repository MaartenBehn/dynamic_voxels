pub const CSG_CHILD_TYPE_NONE: u32 = 0;
pub const CSG_CHILD_TYPE_UNION: u32 = 1;
pub const CSG_CHILD_TYPE_REMOVE: u32 = 2;
pub const CSG_CHILD_TYPE_INTERSECT: u32 = 3;
pub const CSG_CHILD_TYPE_MAT: u32 = 4;
pub const CSG_CHILD_TYPE_BOX: u32 = 5;
pub const CSG_CHILD_TYPE_SPHERE: u32 = 6;
pub const CSG_CHILD_TYPE_VOXEL_GRID: u32 = 7;

pub const CSG_DATA_AABB_SIZE: usize = 6;
pub const CSG_DATA_TRANSFORM_SIZE: usize = 12;

#[derive(Clone, Debug)]
pub struct RenderCSGTree {
    pub data: Vec<u32>
}
