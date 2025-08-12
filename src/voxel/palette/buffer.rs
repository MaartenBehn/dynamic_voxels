use std::sync::atomic::Ordering;

use octa_force::{vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};

use super::{palette::LocalPalette, shared::SharedPalette};


#[derive(Debug)]
pub struct PaletteBuffer {
    pub buffer: Buffer,
    pub palette: SharedPalette,
}

impl PaletteBuffer {
    pub fn new(context: &Context, palette: SharedPalette) -> OctaResult<PaletteBuffer> {
        let buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::TRANSFER_DST, 
            MemoryLocation::GpuOnly,
            (size_of::<u64>() * 256) as _)?;

        Ok(PaletteBuffer {
            buffer,
            palette,
        })
    }

    pub fn update(&self, context: &Context, palette: &SharedPalette) -> OctaResult<()> {
        if palette.changed.load(Ordering::Relaxed) {
            palette.changed.store(false, Ordering::Relaxed);

            let inner = palette.palette.read();
            let packed: Vec<_> = inner.materials.iter().map(|m| m.get_encoded()).collect();

            // TODO Maybe reuse staging buffer?
            context.copy_data_to_gpu_only_buffer(&packed, &self.buffer)
        } else {
            Ok(())
        }
    }
}
