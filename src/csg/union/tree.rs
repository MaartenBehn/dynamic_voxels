use bvh::{bvh::Bvh, flat_bvh::FlatNode};
use octa_force::glam::Mat4;

use crate::{csg::{r#box::CSGBox, sphere::CSGSphere}, util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

#[derive(Debug, Clone)]
pub enum UnionNodeData<M, V: Ve<T, D>, T: Nu, const D: usize> {
    Box(CSGBox<M, V, T, D>),
    Sphere(CSGSphere<M, V, T, D>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

#[derive(Debug, Clone)]
pub struct UnionNode<M, V: Ve<T, D>, T: Nu, const D: usize> {
    pub bh_index: usize,
    pub data: UnionNodeData<M, V, T, D>
}

#[derive(Debug, Clone, Copy)]
pub struct BVHNode<V: Ve<T, D>, T: Nu, const D: usize> {
    pub aabb: AABB<V, T, D>,
    pub exit: usize,
    pub leaf: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Union<M, V: Ve<T, D>, T: Nu, const D: usize> {
    pub nodes: Vec<UnionNode<M, V, T, D>>,
    pub bvh: Vec<BVHNode<V, T, D>>,
    pub changed: bool,
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> Union<M, V, T, D> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            bvh: vec![],
            changed: false, 
        }
    }

    pub fn add_node(&mut self, node: UnionNode<M, V, T, D>) {
        self.nodes.push(node);
        self.changed = true;
    }
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> UnionNode<M, V, T, D> {
    pub fn new(data: UnionNodeData<M, V, T, D>) -> Self {
        Self {
            bh_index: 0,
            data,
        }
    }
}
