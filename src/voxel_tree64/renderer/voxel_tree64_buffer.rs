use octa_force::{glam::UVec3, log::info, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use crate::voxel_tree64::VoxelTree64;
use super::Tree64Renderer;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[repr(C)]
pub struct Tree64Data {
    pub nodes_ptr: u64,
    pub leaf_ptr: u64,
    pub start_index: u32,
    fill_1: u32,
}

#[derive(Debug)]
pub struct VoxelTree64Buffer {
    pub tree: tree64::Tree64<u8>,
    pub nodes_buffer: Buffer,
    pub data_buffer: Buffer,
}

impl VoxelTree64Buffer {
    pub fn get_data(&self) -> Tree64Data {
        Tree64Data {
            start_index: self.tree.root_state().index,
            nodes_ptr: self.nodes_buffer.get_device_address(),
            leaf_ptr: self.data_buffer.get_device_address(),
            fill_1: 0,
        }
    }
}

impl VoxelTree64 {
    pub fn into_buffer(self, context: &Context) -> OctaResult<VoxelTree64Buffer> {
        
        let buffer_size = (size_of::<u8>() * self.tree.nodes.len());
        info!("Tree64 Node Buffer size: {:.04} MB", buffer_size as f32 * 0.000001);

        let nodes_buffer = context.create_gpu_only_buffer_from_data(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            &self.tree.nodes
        )?;
         
        let buffer_size = (size_of::<u8>() * self.tree.data.len());
        info!("Tree64 Data Buffer size: {:.04} MB", buffer_size as f32 * 0.000001);

        let data_buffer = context.create_gpu_only_buffer_from_data(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            &self.tree.data
        )?;

        Ok(VoxelTree64Buffer { 
            tree: self.tree, 
            nodes_buffer, 
            data_buffer, 
        })
    }
}
