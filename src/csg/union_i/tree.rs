use bvh::{bvh::Bvh, flat_bvh::FlatNode};
use octa_force::glam::Mat4;

use crate::{csg::{r#box::CSGBox, sphere::CSGSphere}, util::{aabb3d::AABB, iaabb3d::AABBI}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

pub enum CSGUnionNodeDataI<T> {
    Box(CSGBox<T>),
    Sphere(CSGSphere<T>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

pub struct CSGUnionNodeI<T> {
    pub bh_index: usize,
    pub data: CSGUnionNodeDataI<T>
}

pub struct BVHNodeI {
    pub aabb: AABBI,
    pub exit: usize,
    pub leaf: Option<usize>,
}

pub struct CSGUnionI<T> {
    pub nodes: Vec<CSGUnionNodeI<T>>,
    pub bvh: Vec<BVHNodeI>,
    pub changed: bool,
}

impl<T> CSGUnionI<T> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            bvh: vec![],
            changed: false, 
        }
    }

    pub fn add_node(&mut self, node: CSGUnionNodeI<T>) {
        self.nodes.push(node);
        self.changed = true;
    }
}

impl<T> CSGUnionNodeI<T> {
    pub fn new(data: CSGUnionNodeDataI<T>) -> Self {
        Self {
            bh_index: 0,
            data,
        }
    }
}
