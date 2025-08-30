use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{sphere::CSGSphere, Base}, util::{number::Nu, vector::Ve}, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{Union, UnionNode, UnionNodeData};


impl<M: Base, V: Ve<T, D>, T: Nu, const D: usize> Union<M, V, T, D> {
    pub fn add_sphere(&mut self, center: V::VectorF, radius: f32, mat: M) {
        self.add_node(UnionNode::new_sphere(center, radius, mat));
    }
}

impl<M: Base, V: Ve<T, 3>, T: Nu> Union<M, V, T, 3> {
    pub fn add_disk(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M) {
        self.add_node(UnionNode::new_disk(center, radius, height, mat));
    }

    pub fn add_shared_grid(&mut self, grid: SharedVoxelGrid) {
        self.add_node(UnionNode::new_shared_grid(grid));
    }
}

impl <M: Base, V: Ve<T, D>, T: Nu, const D: usize> UnionNode<M, V, T, D> {
    pub fn new_sphere(center: V::VectorF, radius: f32, mat: M) -> Self {
        UnionNode::new(UnionNodeData::Sphere(CSGSphere::new_sphere(center, radius, mat)))
    }
}

impl <M: Base, V: Ve<T, 3>, T: Nu> UnionNode<M, V, T, 3> {
    pub fn new_disk(center:  V::VectorF, radius: f32, height: f32, mat: M) -> Self {
        UnionNode::new(UnionNodeData::Sphere(CSGSphere::new_disk(center, radius, height, mat)))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        UnionNode::new(UnionNodeData::SharedVoxelGrid(grid))
    }
} 
