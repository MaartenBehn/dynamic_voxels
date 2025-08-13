use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{sphere::CSGSphere, Base}, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGUnion, CSGUnionNode, CSGUnionNodeData};

impl <T: Base + Clone> CSGUnionNode<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        CSGUnionNode::new(CSGUnionNodeData::Sphere(CSGSphere::new_sphere(center, radius)))
    }

    pub fn new_disk(center: Vec3, radius: f32, height: f32) -> Self {
        CSGUnionNode::new(CSGUnionNodeData::Sphere(CSGSphere::new_disk(center, radius, height)))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        CSGUnionNode::new(CSGUnionNodeData::SharedVoxelGrid(grid))
    }
} 
