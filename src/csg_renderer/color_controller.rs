use octa_force::glam::{vec4, Vec4};
use octa_force::vulkan::ash::vk;
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::{Buffer, Context};
use octa_force::OctaResult;

pub const COLOR_BUFFER_SIZE: usize = 256;
pub type Material = u8;
pub const MATERIAL_NONE: u8 = 0;
pub const MATERIAL_BASE: u8 = 1;

pub struct ColorController {
    pub buffer: Buffer,
}

impl ColorController {
    pub fn new(context: &Context) -> OctaResult<Self> {
        let mut colors = [Vec4::ZERO; COLOR_BUFFER_SIZE];
        colors[1] = vec4(1.0, 1.0, 1.0, 1.0);
        colors[2] = vec4(0.0, 0.5, 0.0, 1.0);

        let buffer = context
            .create_gpu_only_buffer_from_data(vk::BufferUsageFlags::UNIFORM_BUFFER, &colors)?;

        Ok(ColorController { buffer })
    }
}

