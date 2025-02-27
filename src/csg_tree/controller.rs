use octa_force::log::info;
use octa_force::vulkan::ash::vk;
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::{Buffer, Context};
use octa_force::OctaResult;

pub const MAX_CSG_TREE_DATA_SIZE: usize = 10000000;
pub struct CSGController {
    pub buffer: Buffer,
}

impl CSGController {
    pub fn new(context: &Context) -> OctaResult<Self> {

        let buffer_size = (size_of::<u32>() * MAX_CSG_TREE_DATA_SIZE);
        info!("CSG Buffer size: {:.04} MB", buffer_size as f32 * 0.000001);

        let buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::CpuToGpu,
            buffer_size as _,
        )?;
        

        Ok(CSGController { buffer })
    }

    pub fn set_data(&self, data: &[u32]) -> OctaResult<()> {
        self.buffer.copy_data_to_buffer(data)
    }
}
