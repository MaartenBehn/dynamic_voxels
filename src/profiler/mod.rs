use log::debug;
use octa_force::glam::{UVec2};
use octa_force::OctaResult;
use octa_force::vulkan::ash::vk;
use octa_force::vulkan::{Buffer, Context, DescriptorPool, WriteDescriptorSet, WriteDescriptorSetKind};
use octa_force::vulkan::gpu_allocator::MemoryLocation;

pub const SCOPES: usize = 20;


pub struct Profiler {
    out_data_len: usize,
    profiler_in_buffer: Buffer,
    profiler_out_buffer: Buffer,

    descriptor_pool: DescriptorPool,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
pub struct ProfilerInData {
    pub active_pixel: u32,
    pub counter: u32,
    pub max_timings: u32,
    pub mode: u32,
}


impl Profiler {
    pub fn new(
        context: &Context,
        res: UVec2,
        num_frames: usize,
    ) -> OctaResult<Profiler> {
        let profiler_in_buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
            size_of::<ProfilerInData>() as _,
        )?;
        
        let out_data_len = SCOPES * 5;
        let profiler_size: usize = size_of::<u32>() * out_data_len;
        debug!("Profiler Buffer size: {} MB", profiler_size as f32 / 1000000.0);
        let profiler_out_buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::GpuToCpu,
            profiler_size as _,
        )?;

        let descriptor_pool = context.create_descriptor_pool(
            num_frames as u32,
            &[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: num_frames as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: num_frames as u32,
                },
            ],
        )?;

        Ok(Profiler {
            profiler_in_buffer,
            profiler_out_buffer,
            descriptor_pool,
            out_data_len,
        })
    }

    pub fn descriptor_layout_bindings(&self) -> [vk::DescriptorSetLayoutBinding; 2] {
        [
            vk::DescriptorSetLayoutBinding {
                binding: 10,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 11,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
        ]
    }

    pub fn write_descriptor_sets(&self) -> [WriteDescriptorSet; 2] {
        [
            WriteDescriptorSet {
                binding: 10,
                kind: WriteDescriptorSetKind::UniformBuffer {
                    buffer: &self.profiler_in_buffer,
                },
            },
            WriteDescriptorSet {
                binding: 11,
                kind: WriteDescriptorSetKind::StorageBuffer {
                    buffer: &self.profiler_out_buffer,
                },
            },
        ]
    }

    pub fn print_result(&self) -> OctaResult<()> {
        let data: Vec<u32> = self.profiler_out_buffer.get_data_from_buffer(self.out_data_len)?;
        
        let num_scopes = self.out_data_len / 5;
        debug!("{} scopes", num_scopes);
        
        let total_start = data[1] as u64 + (data[2] as u64) << 32;
        let total_end = data[3] as u64 + (data[4] as u64) << 32;
        let total_time = total_end - total_start;
        
        for i in 1..num_scopes {
            let counter = data[i * 5];
            let start = data[i * 5 + 1] as u64 + (data[i * 5 + 2] as u64) << 32;
            let end = data[i * 5 + 3] as u64 + (data[i * 5 + 4] as u64) << 32;
            
            if end < start {
                continue
            }
            let percent = ((end - start) as f64 / total_time as f64) * counter as f64 * 100.0;
            
            debug!("{i}: {counter} {percent:0.04}%")
        }

        Ok(())
    }
}