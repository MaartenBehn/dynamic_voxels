use crate::{csg::Base, util::math_config::MC, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGTree, CSGTreeIndex, CSGTreeNode, CSGTreeNodeData};


impl<V: Base, C: MC<D>, const D: usize> CSGTree<V, C, D> {
    
    pub fn union_sphere(&mut self, center: C::VectorF, radius: f32) -> CSGTreeIndex {
        self.union_node_at_root(CSGTreeNode::new_sphere(center, radius))
    }

    pub fn cut_with_sphere(&mut self, center: C::VectorF, radius: f32) -> CSGTreeIndex {
        self.cut_node_at_root(CSGTreeNode::new_sphere(center, radius))
    }

    pub fn union_sphere_at_index(&mut self, center: C::VectorF, radius: f32, index: CSGTreeIndex) -> CSGTreeIndex { 
        self.union_node_at_index(CSGTreeNode::new_sphere(center, radius), index)
    }

    pub fn cut_with_sphere_at_index(&mut self, center: C::VectorF, radius: f32, index: CSGTreeIndex) -> CSGTreeIndex { 
        self.cut_node_at_index(CSGTreeNode::new_sphere(center, radius), index)
    }

}

impl<V: Base, C: MC<3>> CSGTree<V, C, 3> {
    pub fn union_disk(&mut self, center: C::VectorF, radius: f32, height: f32) -> CSGTreeIndex {
        self.union_node_at_root(CSGTreeNode::new_disk(center, radius, height))
    }

    pub fn cut_with_disk(&mut self, center: C::VectorF, radius: f32, height: f32) -> CSGTreeIndex {
        self.cut_node_at_root(CSGTreeNode::new_disk(center, radius, height))
    }

    pub fn union_disk_at_index(&mut self, center: C::VectorF, radius: f32, height: f32, index: CSGTreeIndex) -> CSGTreeIndex {
        self.union_node_at_index(CSGTreeNode::new_disk(center, radius, height), index)
    }

    pub fn cut_with_disk_at_index(&mut self, center: C::VectorF, radius: f32, height: f32, index: CSGTreeIndex) -> CSGTreeIndex {
        self.cut_node_at_index(CSGTreeNode::new_disk(center, radius, height), index)
    }



    pub fn union_shared_grid(&mut self, grid: SharedVoxelGrid) -> CSGTreeIndex {
        self.union_node_at_root(CSGTreeNode::new_shared_grid(grid))
    }

    pub fn cut_with_shared_grid(&mut self, grid: SharedVoxelGrid) -> CSGTreeIndex {
        self.cut_node_at_root(CSGTreeNode::new_shared_grid(grid))
    }

    pub fn union_shared_grid_at_index(&mut self, grid: SharedVoxelGrid, index: CSGTreeIndex) -> CSGTreeIndex {
        self.union_node_at_index(CSGTreeNode::new_shared_grid(grid), index)
    }

    pub fn remove_shared_grid_at_index(&mut self, grid: SharedVoxelGrid, index: CSGTreeIndex) -> CSGTreeIndex {
        self.cut_node_at_index(CSGTreeNode::new_shared_grid(grid), index)
    }
}

