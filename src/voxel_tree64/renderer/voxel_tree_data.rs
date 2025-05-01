use octa_force::glam::UVec3;

use crate::voxel_tree64::VoxelTree64;

use super::Renderer;


#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
pub struct VoxelTreeData {
    pub origin: UVec3,
    pub tree_scale: u32,
    pub nodes_ptr: u64,
    pub leaf_ptr: u64,
}

impl Renderer {
    pub fn new_voxel_tree_data(&self, origin: UVec3, tree: &VoxelTree64) -> VoxelTreeData {
        VoxelTreeData {
            origin,
            tree_scale: 10,
            nodes_ptr: self.tree_node_buffer.get_device_address(),
            leaf_ptr: self.tree_data_buffer.get_device_address(),
        }
    }
}
