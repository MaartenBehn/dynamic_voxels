// based on https://github.com/expenses/tree64 

pub mod node;
pub mod from_voxel_gird;

use node::VoxelDAG64Node;
use octa_force::glam::Vec3;

use crate::multi_data_buffer::{allocated_vec::AllocatedVec};

#[derive(Debug)]
pub struct VoxelDAG64 {
    pub nodes: AllocatedVec<VoxelDAG64Node>,
    pub data: AllocatedVec<u8>,

    pub levels: u8,
    pub root_index: u32,
}

impl VoxelDAG64 {
    pub fn get_size(&self) -> Vec3 {
        let size = self.get_size_u32();
        Vec3::splat(size as f32)
    }

    pub fn get_size_u32(&self) -> u32 {
        4_u32.pow(self.levels as u32 - 1)
    }
}
