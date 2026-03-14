use octa_force::glam::Mat4;

use crate::{csg::{Base, primitves::{CSGPrimitive, r#box::CSGBox, cylinder::CSGCylinder, sphere::CSGSphere}}, util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{remove::CSGTreeRemove, union::CSGTreeUnion};

pub type CSGTreeIndex = usize; 
pub const CSG_TREE_INDEX_INVALID: CSGTreeIndex = CSGTreeIndex::MAX;

#[derive(Debug, Clone)]
pub enum CSGTreeNodeData<M, V: Ve<T, D>, T: Nu, const D: usize> {
    Union(CSGTreeUnion<V, T, D>),
    Cut(CSGTreeRemove),
   
    None,
    Box(CSGPrimitive<CSGBox, M, V, T, D>),
    Sphere(CSGPrimitive<CSGSphere, M, V, T, D>),
    Cylinder(CSGPrimitive<CSGCylinder, M, V, T, D>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

#[derive(Debug, Clone)]
pub struct CSGTreeNode<M, V: Ve<T, D>, T: Nu, const D: usize> {
    pub data: CSGTreeNodeData<M, V, T, D>,
    pub parent: CSGTreeIndex,
}

#[derive(Debug, Clone, Default)]
pub struct CSGTree<M, V: Ve<T, D>, T: Nu, const D: usize> {
    pub nodes: Vec<CSGTreeNode<M, V, T, D>>,
    pub root: CSGTreeIndex,
    pub needs_bounds_recompute: bool,
    pub changed_bounds: AABB<V, T, D>,
}



impl<M, V: Ve<T, D>, T: Nu, const D: usize> CSGTreeNode<M, V, T, D> {
    pub fn new(data: CSGTreeNodeData<M, V, T, D>, parent: CSGTreeIndex) -> Self {
        Self {
            data,
            parent,
        }
    }
}
