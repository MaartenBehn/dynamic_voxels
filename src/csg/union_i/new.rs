use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{sphere::CSGSphere, Base}, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGUnionI, CSGUnionNodeI, CSGUnionNodeDataI};

impl <T: Base + Clone> CSGUnionNodeI<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        CSGUnionNodeI::new(CSGUnionNodeDataI::Sphere(CSGSphere::new_sphere(center, radius)))
    }

    pub fn new_disk(center: Vec3, radius: f32, height: f32) -> Self {
        CSGUnionNodeI::new(CSGUnionNodeDataI::Sphere(CSGSphere::new_disk(center, radius, height)))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        CSGUnionNodeI::new(CSGUnionNodeDataI::SharedVoxelGrid(grid))
    }
} 
