use octa_force::{vulkan::{ash::vk, Buffer, Context}, OctaResult};


#[derive(Debug)]
pub enum Key {
    RayCasts,
    TraversalIters,
    ClocksPerRay,

    ReservedStart_ = 10,
    FrameStartTS, FrameEndTS,
    SyncStartTS, SyncEndTS,
    Count_,
}

#[derive(Debug)]
pub struct FramePerfStats {
    pub buffer: Buffer
}

#[derive(Debug)]
pub struct FramePerfStatsData {
    counters: [u64; 16],
    ray_cast_iters_histogram: [u32; 32],
}

impl FramePerfStats {
    pub fn new(context: &Context) -> OctaResult<Self> {
        
        let buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            octa_force::vulkan::gpu_allocator::MemoryLocation::GpuToCpu,
            size_of::<FramePerfStatsData>() as _
        )?;

        Ok(Self {
            buffer
        })
    }
}



