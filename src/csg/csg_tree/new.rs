use octa_force::glam::{vec3, Mat4, Quat, Vec3};
use slotmap::Key;

use crate::{csg::{sphere::CSGSphere, Base}, util::aabb3d::AABB3, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGNode, CSGNodeData, CSGTree, CSGTreeKey};


impl<T: Base + Clone> CSGTree<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        CSGTree::from_node(CSGNode::new_sphere(center, radius))
    }

    pub fn new_disk(center: Vec3, radius: f32, height: f32) -> Self {
        CSGTree::from_node(CSGNode::new_disk(center, radius, height))
    } 
}

impl <T: Base + Clone> CSGNode<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        CSGNode::new(CSGNodeData::Sphere(CSGSphere::new_sphere(center, radius)))
    }

    pub fn new_disk(center: Vec3, radius: f32, height: f32) -> Self {
        CSGNode::new(CSGNodeData::Sphere(CSGSphere::new_disk(center, radius, height)))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        CSGNode::new(CSGNodeData::SharedVoxelGrid(grid))
    }
} 


