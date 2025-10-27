use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{r#box::CSGBox, sphere::CSGSphere, Base}, util::{number::Nu, vector::Ve}, voxel::grid::shared::SharedVoxelGrid};

use super::{remove::CSGTreeRemove, tree::{CSGTree, CSGTreeIndex, CSGTreeNode, CSGTreeNodeData, CSG_TREE_INDEX_INVALID}, union::CSGTreeUnion};


impl<M: Base, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {
    pub fn new_sphere(center: V, radius: T, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_sphere(center.to_vecf(), radius.to_f32(), mat))
    }

    pub fn new_box(center: V, size: V, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_box(center.to_vecf(), size.to_vecf(), mat))
    }

    pub fn new_sphere_float(center: V::VectorF, radius: f32, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_sphere(center, radius, mat))
    }

    pub fn new_box_float(center: V::VectorF, size: V::VectorF, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_box(center, size, mat))
    }

}

impl<M: Base, V: Ve<T, 3>, T: Nu> CSGTree<M, V, T, 3> {
    pub fn new_disk(center: V::VectorF, radius: f32, height: f32, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_disk(center, radius, height, mat))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        Self::from_node(CSGTreeNode::new_shared_grid(grid))
    }
}

impl <M: Base, V: Ve<T, D>, T: Nu, const D: usize> CSGTreeNode<M, V, T, D> {
    pub fn new_none() -> Self {
        CSGTreeNode::new(CSGTreeNodeData::None, CSG_TREE_INDEX_INVALID)
    }

    pub fn new_sphere(center: V::VectorF, radius: f32, mat: M) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Sphere(CSGSphere::new_sphere(center, radius, mat)), CSG_TREE_INDEX_INVALID)
    }

    pub fn new_box(center: V::VectorF, size: V::VectorF, mat: M) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Box(CSGBox::new(center, size, mat)), CSG_TREE_INDEX_INVALID)
    }

    pub fn new_union(nodes: Vec<CSGTreeIndex>) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Union(CSGTreeUnion::new(nodes)), CSG_TREE_INDEX_INVALID)
    }
    
    pub fn new_cut(base: CSGTreeIndex, cut: CSGTreeIndex) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Cut(CSGTreeRemove::new(base, cut)), CSG_TREE_INDEX_INVALID)
    }
}

impl <M: Base, V: Ve<T, 3>, T: Nu> CSGTreeNode<M, V, T, 3> {
    pub fn new_disk(center:  V::VectorF, radius: f32, height: f32, mat: M) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Sphere(CSGSphere::new_disk(center, radius, height, mat)), CSG_TREE_INDEX_INVALID)
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::SharedVoxelGrid(grid), CSG_TREE_INDEX_INVALID)
    }
} 
