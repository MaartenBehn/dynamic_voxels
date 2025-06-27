// based on https://github.com/expenses/tree64 

pub mod node;
pub mod from_voxel_gird;

use node::VoxelDAG64Node;

pub struct VoxelDAG64 {
    pub nodes: Vec<VoxelDAG64Node>,
    pub data: Vec<u8>,

    pub levels: usize,
    pub root_index: usize,
}

