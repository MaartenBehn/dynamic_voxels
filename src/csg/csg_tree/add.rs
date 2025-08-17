use crate::{csg::Base, util::math_config::MC, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGTree, CSGTreeIndex, CSGTreeNode, CSGTreeNodeData, CutResult, UnionResult};


impl<V: Base, C: MC<D>, const D: usize> CSGTree<V, C, D> {
    
    pub fn union_sphere(&mut self, center: C::VectorF, radius: f32, mat: V) -> UnionResult {
        self.union_node_at_root(CSGTreeNode::new_sphere(center, radius, mat))
    }

    pub fn cut_with_sphere(&mut self, center: C::VectorF, radius: f32, mat: V) -> CutResult {
        self.cut_node_at_root(CSGTreeNode::new_sphere(center, radius, mat))
    }

    pub fn union_sphere_at_index(&mut self, center: C::VectorF, radius: f32, mat: V, index: CSGTreeIndex) -> UnionResult { 
        self.union_node_at_index(CSGTreeNode::new_sphere(center, radius, mat), index)
    }

    pub fn cut_with_sphere_at_index(&mut self, center: C::VectorF, radius: f32, mat: V, index: CSGTreeIndex) -> CutResult { 
        self.cut_node_at_index(CSGTreeNode::new_sphere(center, radius, mat), index)
    }

}

impl<V: Base, C: MC<3>> CSGTree<V, C, 3> {
    pub fn union_disk(&mut self, center: C::VectorF, radius: f32, height: f32, mat: V) -> UnionResult {
        self.union_node_at_root(CSGTreeNode::new_disk(center, radius, height, mat))
    }

    pub fn cut_with_disk(&mut self, center: C::VectorF, radius: f32, height: f32, mat: V) -> CutResult {
        self.cut_node_at_root(CSGTreeNode::new_disk(center, radius, height, mat))
    }

    pub fn union_disk_at_index(&mut self, center: C::VectorF, radius: f32, height: f32, mat: V, index: CSGTreeIndex) -> UnionResult {
        self.union_node_at_index(CSGTreeNode::new_disk(center, radius, height, mat), index)
    }

    pub fn cut_with_disk_at_index(&mut self, center: C::VectorF, radius: f32, height: f32, mat: V, index: CSGTreeIndex) -> CutResult {
        self.cut_node_at_index(CSGTreeNode::new_disk(center, radius, height, mat), index)
    }



    pub fn union_shared_grid(&mut self, grid: SharedVoxelGrid) -> UnionResult {
        self.union_node_at_root(CSGTreeNode::new_shared_grid(grid))
    }

    pub fn cut_with_shared_grid(&mut self, grid: SharedVoxelGrid) -> CutResult {
        self.cut_node_at_root(CSGTreeNode::new_shared_grid(grid))
    }

    pub fn union_shared_grid_at_index(&mut self, grid: SharedVoxelGrid, index: CSGTreeIndex) -> UnionResult {
        self.union_node_at_index(CSGTreeNode::new_shared_grid(grid), index)
    }

    pub fn remove_shared_grid_at_index(&mut self, grid: SharedVoxelGrid, index: CSGTreeIndex) -> CutResult {
        self.cut_node_at_index(CSGTreeNode::new_shared_grid(grid), index)
    }
}

