use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{sphere::CSGSphere, Base}, util::math_config::MC, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{Union, UnionNode, UnionNodeData};


impl<V: Base, C: MC<D>, const D: usize> Union<V, C, D> {
    pub fn add_sphere(&mut self, center: C::VectorF, radius: f32, mat: V) {
        self.add_node(UnionNode::new_sphere(center, radius, mat));
    }
}

impl<V: Base, C: MC<3>> Union<V, C, 3> {
    pub fn add_disk(&mut self, center: C::VectorF, radius: f32, height: f32, mat: V) {
        self.add_node(UnionNode::new_disk(center, radius, height, mat));
    }

    pub fn add_shared_grid(&mut self, grid: SharedVoxelGrid) {
        self.add_node(UnionNode::new_shared_grid(grid));
    }
}

impl <V: Base, C: MC<D>, const D: usize> UnionNode<V, C, D> {
    pub fn new_sphere(center: C::VectorF, radius: f32, mat: V) -> Self {
        UnionNode::new(UnionNodeData::Sphere(CSGSphere::new_sphere(center, radius, mat)))
    }
}

impl <V: Base, C: MC<3>> UnionNode<V, C, 3> {
    pub fn new_disk(center:  C::VectorF, radius: f32, height: f32, mat: V) -> Self {
        UnionNode::new(UnionNodeData::Sphere(CSGSphere::new_disk(center, radius, height, mat)))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        UnionNode::new(UnionNodeData::SharedVoxelGrid(grid))
    }
} 
