use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{sphere::CSGSphere, Base}, util::math_config::MC, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGUnion, CSGUnionNode, CSGUnionNodeData};


impl<V: Base, C: MC<D>, const D: usize> CSGUnion<V, C, D> {
    pub fn add_sphere(&mut self, center: C::VectorF, radius: f32) {
        self.add_node(CSGUnionNode::new_sphere(center, radius));
    }
}

impl<V: Base, C: MC<3>> CSGUnion<V, C, 3> {
    pub fn add_disk(&mut self, center: C::VectorF, radius: f32, height: f32) {
        self.add_node(CSGUnionNode::new_disk(center, radius, height));
    }

    pub fn add_shared_grid(&mut self, grid: SharedVoxelGrid) {
        self.add_node(CSGUnionNode::new_shared_grid(grid));
    }
}

impl <V: Base, C: MC<D>, const D: usize> CSGUnionNode<V, C, D> {
    pub fn new_sphere(center: C::VectorF, radius: f32) -> Self {
        CSGUnionNode::new(CSGUnionNodeData::Sphere(CSGSphere::new_sphere(center, radius)))
    }
}

impl <V: Base, C: MC<3>> CSGUnionNode<V, C, 3> {
    pub fn new_disk(center:  C::VectorF, radius: f32, height: f32) -> Self {
        CSGUnionNode::new(CSGUnionNodeData::Sphere(CSGSphere::new_disk(center, radius, height)))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        CSGUnionNode::new(CSGUnionNodeData::SharedVoxelGrid(grid))
    }
} 
