use octa_force::glam::{IVec3, Mat4};
use slotmap::{new_key_type, SlotMap};

use crate::{aabb::AABB, csg_renderer::color_controller::Material, voxel::grid::VoxelGrid};

new_key_type! { pub struct SlotMapCSGTreeKey; }

#[derive(Clone, Debug)]
pub struct SlotMapCSGTree {
    pub nodes: SlotMap<SlotMapCSGTreeKey, SlotMapCSGNode>,
    pub root_node: SlotMapCSGTreeKey,
}

#[derive(Clone, Debug)]
pub struct SlotMapCSGNode {
    pub data: SlotMapCSGNodeData,
    pub aabb: AABB,
    pub parent: SlotMapCSGTreeKey,
}

#[derive(Clone, Debug)]
pub enum SlotMapCSGNodeData {
    Union(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Remove(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Intersect(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Mat(Mat4, SlotMapCSGTreeKey),
    Box(Mat4, Material),
    Sphere(Mat4, Material),
    All(Material),
    VoxelGrid(VoxelGrid, IVec3),
}

