use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{sphere::CSGSphere, Base}, util::math_config::MC, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGTreeNode, CSGTreeNodeData, CSGTree};


impl<V: Base, C: MC<D>, const D: usize> CSGTree<V, C, D> {
    pub fn add_sphere(&mut self, center: C::VectorF, radius: f32) {
        self.add_node(CSGTreeNode::new_sphere(center, radius));
    }
}

impl<V: Base, C: MC<3>> CSGTree<V, C, 3> {
    pub fn add_disk(&mut self, center: C::VectorF, radius: f32, height: f32) {
        self.add_node(CSGTreeNode::new_disk(center, radius, height));
    }

    pub fn add_shared_grid(&mut self, grid: SharedVoxelGrid) {
        self.add_node(CSGTreeNode::new_shared_grid(grid));
    }
}

impl <V: Base, C: MC<D>, const D: usize> CSGTreeNode<V, C, D> {
    pub fn new_sphere(center: C::VectorF, radius: f32) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Sphere(CSGSphere::new_sphere(center, radius)))
    }
}

impl <V: Base, C: MC<3>> CSGTreeNode<V, C, 3> {
    pub fn new_disk(center:  C::VectorF, radius: f32, height: f32) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Sphere(CSGSphere::new_disk(center, radius, height)))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::SharedVoxelGrid(grid))
    }
} 
