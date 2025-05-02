use octa_force::{glam::{vec4, Vec4}, vulkan::{ash::vk, Buffer, Context}, OctaResult};


pub struct Palette {
    pub buffer: Buffer,
}

impl Palette {
    pub fn new(context: &Context) -> OctaResult<Self> {
        let mut colors = [Vec4::ZERO; 256];
        colors[1] = vec4(1.0, 1.0, 1.0, 1.0);
        colors[2] = vec4(0.0, 0.5, 0.0, 1.0);

        let buffer = context
            .create_gpu_only_buffer_from_data(vk::BufferUsageFlags::UNIFORM_BUFFER, &colors)?;

        Ok(Self { buffer })

    }
}
