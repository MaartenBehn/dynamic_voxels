use bvh::{bvh::Bvh, flat_bvh::FlatNode};
use octa_force::glam::Mat4;

use crate::{csg::{r#box::CSGBox, sphere::CSGSphere}, util::{aabb::AABB, aabb3d::AABB3, math_config::MC}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

#[derive(Debug, Clone)]
pub enum CSGUnionNodeData<V, C: MC<D>, const D: usize> {
    Box(CSGBox<V, C, D>),
    Sphere(CSGSphere<V, C, D>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

#[derive(Debug, Clone)]
pub struct CSGUnionNode<V, C: MC<D>, const D: usize> {
    pub bh_index: usize,
    pub data: CSGUnionNodeData<V, C, D>
}

#[derive(Debug, Clone, Copy)]
pub struct BVHNode<C: MC<D>, const D: usize> {
    pub aabb: AABB<C, D>,
    pub exit: usize,
    pub leaf: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct CSGUnion<V, C: MC<D>, const D: usize> {
    pub nodes: Vec<CSGUnionNode<V, C, D>>,
    pub bvh: Vec<BVHNode<C, D>>,
    pub changed: bool,
}

impl<V, C: MC<D>, const D: usize> CSGUnion<V, C, D> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            bvh: vec![],
            changed: false, 
        }
    }

    pub fn add_node(&mut self, node: CSGUnionNode<V, C, D>) {
        self.nodes.push(node);
        self.changed = true;
    }
}

impl<V, C: MC<D>, const D: usize> CSGUnionNode<V, C, D> {
    pub fn new(data: CSGUnionNodeData<V, C, D>) -> Self {
        Self {
            bh_index: 0,
            data,
        }
    }
}
