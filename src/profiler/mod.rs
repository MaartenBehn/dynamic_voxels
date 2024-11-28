use std::iter;
use std::ops::Index;
use log::debug;
use octa_force::glam::{UVec2, Vec3};
use octa_force::OctaResult;
use octa_force::vulkan::ash::vk;
use octa_force::vulkan::{Buffer, Context, DescriptorPool, WriteDescriptorSet, WriteDescriptorSetKind};
use octa_force::vulkan::gpu_allocator::MemoryLocation;

pub const MAX_PROFILE_TIMINGS: usize = 1000;

pub struct Profiler {
    num_timings: usize,
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

        let num_timings = MAX_PROFILE_TIMINGS;
        let out_data_len = num_timings * 3 + 1;
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
            num_timings,
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
        if data[0] <= 0 {
            return Ok(())
        }
        
        let num_timings = ((data[0] - 1) / 3) as usize;
        debug!("{} timings", num_timings);
        
        let mut timings = Vec::with_capacity(num_timings);
        for i in 0..num_timings {
            let id = data[i * 3 + 1];
            let timing = data[i * 3 + 2] as u64 + (data[i * 3 + 3] as u64) << 32;

            timings.push((id, timing));
        }
        
        let total_diff = (timings[num_timings - 1].1 - timings[0].1) as f64;
        let mut level_start = vec![];
        let mut last_timing = 0;
        let mut id_sums = vec![];
        
        for i in 0..num_timings {
            let id = timings[i].0 as usize;
            let timing = timings[i].1;
            
            let level = (id / 10) as usize;
            let in_level = (id % 10) as usize;
            let padding = iter::repeat(" ").take(level).collect::<String>();
            
            let level_percent = if in_level == 0 {
                if level_start.len() <= level {
                    level_start.resize(level + 1, 0);
                }
                
                level_start[level] = timing;
                0.0
            } else {
                let level_start = level_start[level];
                ((timing - level_start) as f64 / total_diff) * 100.0
            };
            
            let last_percent = (timing - last_timing) as f64 / total_diff * 100.0;
            
            debug!("{padding} {id} {level_percent:0.2}% {last_percent:0.2}%");

            last_timing = timing;

            if id_sums.len() <= id {
                id_sums.resize(id + 1, 0.0);
            }

            id_sums[id] += last_percent
        }
        
        for (id, sum) in id_sums.into_iter().enumerate() {
            if id == 0 || sum == 0.0 {
                continue
            }

            debug!("{id} {sum:0.2}%");
        }

        Ok(())
    }
}