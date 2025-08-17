use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{sphere::CSGSphere, Base}, util::math_config::MC, voxel::grid::shared::SharedVoxelGrid};

use super::{remove::CSGTreeRemove, tree::{CSGTree, CSGTreeIndex, CSGTreeNode, CSGTreeNodeData, CSG_TREE_INDEX_INVALID}, union::CSGTreeUnion};


impl<V: Base, C: MC<D>, const D: usize> CSGTree<V, C, D> {
    pub fn new_sphere(center: C::VectorF, radius: f32, mat: V) -> Self {
        Self::from_node(CSGTreeNode::new_sphere(center, radius, mat))
    }
}

impl<V: Base, C: MC<3>> CSGTree<V, C, 3> {
    pub fn new_disk(center: C::VectorF, radius: f32, height: f32, mat: V) -> Self {
        Self::from_node(CSGTreeNode::new_disk(center, radius, height, mat))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        Self::from_node(CSGTreeNode::new_shared_grid(grid))
    }
}

impl <V: Base, C: MC<D>, const D: usize> CSGTreeNode<V, C, D> {
    pub fn new_sphere(center: C::VectorF, radius: f32, mat: V) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Sphere(CSGSphere::new_sphere(center, radius, mat)), CSG_TREE_INDEX_INVALID)
    }

    pub fn new_union(nodes: Vec<CSGTreeIndex>) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Union(CSGTreeUnion::new(nodes)), CSG_TREE_INDEX_INVALID)
    }
    
    pub fn new_remove(base: CSGTreeIndex, remove: CSGTreeIndex) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Remove(CSGTreeRemove::new(base, remove)), CSG_TREE_INDEX_INVALID)
    }
}

impl <V: Base, C: MC<3>> CSGTreeNode<V, C, 3> {
    pub fn new_disk(center:  C::VectorF, radius: f32, height: f32, mat: V) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Sphere(CSGSphere::new_disk(center, radius, height, mat)), CSG_TREE_INDEX_INVALID)
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::SharedVoxelGrid(grid), CSG_TREE_INDEX_INVALID)
    }
} 
