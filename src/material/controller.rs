use crate::buddy_controller::BuddyBufferAllocator;
use crate::material::voxels::VoxelField;
use octa_force::vulkan::ash::vk;
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::{Buffer, Context};
use octa_force::OctaResult;

const INITIAL_MATERIAL_BUFFER_SIZE: usize = 1024;

pub struct MaterialController {
    pub buffer: Buffer,
    pub allocator: BuddyBufferAllocator,
}

impl MaterialController {
    pub fn new(context: &Context) -> OctaResult<Self> {
        let buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
            (size_of::<u32>() * INITIAL_MATERIAL_BUFFER_SIZE) as _,
        )?;

        let allocator = BuddyBufferAllocator::new(INITIAL_MATERIAL_BUFFER_SIZE);

        Ok(MaterialController { buffer, allocator })
    }

    pub fn push_voxel_field(&self, voxel_field: &VoxelField) -> OctaResult<()> {
        self.buffer.copy_data_to_buffer_complex(
            &voxel_field.data,
            voxel_field.buffer_start,
            align_of::<u8>(),
        )
    }

    pub fn allocate_voxel_field(&mut self, voxel_field: &mut VoxelField) -> OctaResult<()> {
        let (start, _) = self.allocator.alloc(voxel_field.buffer_size)?;
        voxel_field.buffer_start = start;

        Ok(())
    }

    pub fn deallocate_voxel_field(&mut self, voxel_field: &VoxelField) -> OctaResult<()> {
        self.allocator.dealloc(voxel_field.buffer_start)?;

        Ok(())
    }
}

