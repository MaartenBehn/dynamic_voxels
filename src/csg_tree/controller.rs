use octa_force::vulkan::ash::vk;
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::{Buffer, Context};
use octa_force::OctaResult;

pub const MAX_CSG_TREE_DATA_SIZE: usize = 16384;
pub struct CSGController {
    pub buffer: Buffer,
}

impl CSGController {
    pub fn new(context: &Context) -> OctaResult<Self> {
        let buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
            (size_of::<u32>() * MAX_CSG_TREE_DATA_SIZE) as _,
        )?;
        

        Ok(CSGController { buffer })
    }

    pub fn set_data(&self, data: &[u32]) -> OctaResult<()> {
        self.buffer.copy_data_to_buffer(data)
    }
}
