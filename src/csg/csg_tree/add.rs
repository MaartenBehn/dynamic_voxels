use crate::{csg::Base, util::{number::Nu, vector::Ve}, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGTree, CSGTreeIndex, CSGTreeNode, CSGTreeNodeData, CutResult, UnionResult};


impl<M: Base, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {

    pub fn add_sphere(&mut self, center: V::VectorF, radius: f32, mat: M) -> usize {
        self.add_node(CSGTreeNode::new_sphere(center, radius, mat)) 
    }
    
    pub fn union_sphere(&mut self, center: V::VectorF, radius: f32, mat: M) -> UnionResult {
        self.union_at_root(&[CSGTreeNode::new_sphere(center, radius, mat)], 0)
    }

    pub fn cut_with_sphere(&mut self, center: V::VectorF, radius: f32, mat: M) -> CutResult {
        self.cut_at_root(&[CSGTreeNode::new_sphere(center, radius, mat)], 0)
    }

    pub fn union_sphere_at_index(&mut self, center: V::VectorF, radius: f32, mat: M, index: CSGTreeIndex) -> UnionResult { 
        self.union_at_index(index, &[CSGTreeNode::new_sphere(center, radius, mat)], 0)
    }

    pub fn cut_with_sphere_at_index(&mut self, center: V::VectorF, radius: f32, mat: M, index: CSGTreeIndex) -> CutResult { 
        self.cut_at_index(index, &[CSGTreeNode::new_sphere(center, radius, mat)], 0)
    }

    pub fn add_box(&mut self, center: V::VectorF, size: V::VectorF, mat: M) -> usize {
        self.add_node(CSGTreeNode::new_box(center, size, mat)) 
    }

    pub fn union_box_at_index(&mut self, center: V::VectorF, size: V::VectorF, mat: M, index: CSGTreeIndex) -> UnionResult { 
        self.union_at_index(index, &[CSGTreeNode::new_box(center, size, mat)], 0)
    }

}

impl<M: Base, V: Ve<T, 3>, T: Nu> CSGTree<M, V, T, 3> {
    pub fn add_disk(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M) -> usize {
        self.add_node(CSGTreeNode::new_disk(center, radius, height, mat)) 
    }

    pub fn union_disk(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M) -> UnionResult {
        self.union_at_root(&[CSGTreeNode::new_disk(center, radius, height, mat)], 0)
    }

    pub fn cut_with_disk(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M) -> CutResult {
        self.cut_at_root(&[CSGTreeNode::new_disk(center, radius, height, mat)], 0)
    }

    pub fn union_disk_at_index(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M, index: CSGTreeIndex) -> UnionResult {
        self.union_at_index(index, &[CSGTreeNode::new_disk(center, radius, height, mat)], 0)
    }

    pub fn cut_with_disk_at_index(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M, index: CSGTreeIndex) -> CutResult {
        self.cut_at_index(index, &[CSGTreeNode::new_disk(center, radius, height, mat)], 0)
    }

    pub fn add_shared_grid(&mut self, grid: SharedVoxelGrid) -> usize {
        self.add_node(CSGTreeNode::new_shared_grid(grid)) 
    }

    pub fn union_shared_grid(&mut self, grid: SharedVoxelGrid) -> UnionResult {
        self.union_at_root(&[CSGTreeNode::new_shared_grid(grid)], 0)
    }

    pub fn cut_with_shared_grid(&mut self, grid: SharedVoxelGrid) -> CutResult {
        self.cut_at_root(&[CSGTreeNode::new_shared_grid(grid)], 0)
    }

    pub fn union_shared_grid_at_index(&mut self, grid: SharedVoxelGrid, index: CSGTreeIndex) -> UnionResult {
        self.union_at_index(index, &[CSGTreeNode::new_shared_grid(grid)], 0)
    }

    pub fn remove_shared_grid_at_index(&mut self, grid: SharedVoxelGrid, index: CSGTreeIndex) -> CutResult {
        self.cut_at_index(index, &[CSGTreeNode::new_shared_grid(grid)], 0)
    }
}

