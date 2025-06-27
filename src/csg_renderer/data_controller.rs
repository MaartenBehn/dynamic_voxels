use octa_force::log::info;
use octa_force::vulkan::ash::vk;
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::{Buffer, Context};
use octa_force::OctaResult;

use crate::render_csg_tree::base::RenderCSGTree;

pub const MAX_DATA_BUFFER_SIZE: usize = 10000000;

#[derive(Debug)]
pub struct DataController {
    pub buffer: Buffer,
}

impl DataController {
    pub fn new(context: &Context) -> OctaResult<Self> {

        let buffer_size = (size_of::<u32>() * MAX_DATA_BUFFER_SIZE);
        info!("Data Buffer size: {:.04} MB", buffer_size as f32 * 0.000001);

        let buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::CpuToGpu,
            buffer_size as _,
        )?;
        

        Ok(DataController { buffer })
    }

    pub fn set_render_csg_tree(&self, tree: &RenderCSGTree) {
        self.buffer.copy_data_to_buffer(&tree.data);
    }
}
