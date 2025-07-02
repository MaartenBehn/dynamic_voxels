// based on https://github.com/expenses/tree64 

pub mod node;
pub mod from_voxel_gird;

use node::VoxelDAG64Node;
use octa_force::{glam::Vec3, log::{debug, info}};

use crate::{multi_data_buffer::{allocated_vec::AllocatedVec, cache_allocated_vec::CacheAllocatedVec, full_search_allocated_vec::FullSearchAllocatedVec, kmp_search_allocated_vec::KmpSearchAllocatedVec}, util::to_mb};

#[derive(Debug)]
pub struct VoxelDAG64 {
    pub nodes: CacheAllocatedVec<VoxelDAG64Node>,
    pub data: CacheAllocatedVec<u8>,

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

    pub fn print_memory_info(&self) { 
        info!("VoxelDAG64: nodes {} MB over {} blocks, data {} MB over {} blocks", 
            to_mb(self.nodes.get_memory_size()),
            self.nodes.get_num_allocations(),
            to_mb(self.data.get_memory_size()),
            self.data.get_num_allocations()
        );
    }
}
