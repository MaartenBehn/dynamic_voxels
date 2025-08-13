use bvh::{bvh::Bvh, flat_bvh::FlatNode};
use octa_force::glam::Mat4;

use crate::{csg::{r#box::CSGBox, sphere::CSGSphere}, util::aabb3d::AABB, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

pub enum CSGUnionNodeData<T> {
    Box(CSGBox<T>),
    Sphere(CSGSphere<T>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

pub struct CSGUnionNode<T> {
    pub bh_index: usize,
    pub data: CSGUnionNodeData<T>
}

pub struct BVHNode {
    pub aabb: AABB,
    pub exit: usize,
    pub leaf: Option<usize>,
}

pub struct CSGUnion<T> {
    pub nodes: Vec<CSGUnionNode<T>>,
    pub bvh: Vec<BVHNode>,
    pub changed: bool,
}

impl<T> CSGUnion<T> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            bvh: vec![],
            changed: false, 
        }
    }

    pub fn add_node(&mut self, node: CSGUnionNode<T>) {
        self.nodes.push(node);
        self.changed = true;
    }
}

impl<T> CSGUnionNode<T> {
    pub fn new(data: CSGUnionNodeData<T>) -> Self {
        Self {
            bh_index: 0,
            data,
        }
    }
}
