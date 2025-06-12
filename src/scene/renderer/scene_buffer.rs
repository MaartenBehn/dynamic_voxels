use octa_force::{glam::{Mat4, Vec3}, log::info, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};

use crate::{buddy_controller::BuddyBufferAllocator, scene::Scene};


pub struct SceneBuffer {
    pub buffer: Buffer,
    pub allocator: BuddyBufferAllocator,
    pub scene: Scene,
}

pub struct SceneObjectData {
    min: Vec3,
    data_1: u32,
    max: Vec3,  
    data_2: u32,
}

pub struct Tree64SceneObjectData {
    mat: Mat4,
    nodes_index: u32,
    leafs_index: u32,
    start_index: u32,
}

impl SceneBuffer {
    pub fn new(context: &Context, scene: Scene) -> OctaResult<Self> {
        let buffer_size = 2_usize.pow(4);
        info!("Scene Buffer size: {:.04} MB", buffer_size as f32 * 0.000001);

        let buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            MemoryLocation::CpuToGpu,
            buffer_size as _,
        )?;

        let allocator = BuddyBufferAllocator::new(buffer_size);
         
        Ok(Self {
            buffer,
            allocator,
            scene,
        })
    }


}
