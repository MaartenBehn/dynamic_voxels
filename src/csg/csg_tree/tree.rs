use bvh::{bvh::Bvh, flat_bvh::FlatNode};
use octa_force::glam::Mat4;

use crate::{csg::{r#box::CSGBox, sphere::CSGSphere}, util::{aabb::AABB, math_config::MC}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{remove::CSGTreeRemove, union::CSGTreeUnion};

pub type CSGTreeIndex = usize; 

#[derive(Debug, Clone)]
pub enum CSGTreeNodeData<V, C: MC<D>, const D: usize> {
    Union(CSGTreeUnion<C, D>),
    Remove(CSGTreeRemove),
    
    Box(CSGBox<V, C, D>),
    Sphere(CSGSphere<V, C, D>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

#[derive(Debug, Clone)]
pub struct CSGTreeNode<V, C: MC<D>, const D: usize> {
    pub data: CSGTreeNodeData<V, C, D>
}

#[derive(Debug, Clone, Default)]
pub struct CSGTree<V, C: MC<D>, const D: usize> {
    pub nodes: Vec<CSGTreeNode<V, C, D>>,
    pub changed: bool,
    pub root: CSGTreeIndex,
}

impl<V, C: MC<D>, const D: usize> CSGTree<V, C, D> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            changed: false,
            root: 0,
        }
    }

    pub fn add_node(&mut self, node: CSGTreeNode<V, C, D>) {
        self.nodes.push(node);
        self.changed = true;
    }
}

impl<V, C: MC<D>, const D: usize> CSGTreeNode<V, C, D> {
    pub fn new(data: CSGTreeNodeData<V, C, D>) -> Self {
        Self {
            data,
        }
    }
}
