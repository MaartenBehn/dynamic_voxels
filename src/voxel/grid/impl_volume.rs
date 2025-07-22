use octa_force::glam::{UVec3, Vec3, Vec3A};

use crate::{util::{aabb3d::AABB, math::to_1d}, volume::{VolumeBounds, VolumeQureyPosValue}};

use super::VoxelGrid;

impl VolumeBounds for VoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB {
        AABB::new(
            Vec3::ZERO,
            self.size.as_vec3())
    }
}

impl VolumeQureyPosValue for VoxelGrid {
    fn get_value_u(&self, pos: UVec3) -> u8 {
        if pos.cmpge(self.size).any() {
            return 0;
        }

        self.data[to_1d(pos, self.size)]
    }

    fn get_value_a(&self, pos: Vec3A) -> u8 {
        self.get_value_u(pos.as_uvec3())
    }

    fn get_value(&self, pos: Vec3) -> u8 {
        self.get_value_u(pos.as_uvec3())
    }
}
