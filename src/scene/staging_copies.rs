use octa_force::vulkan::{Buffer, Context, ash::vk};

use crate::scene::worker::SceneWorker;

pub struct SceneStagingBuilder {
    staging: SceneStaging,
    offset: usize,
    optimal_alignment: OptimalBufferCopyAlligment,
}

#[derive(Debug)]
pub struct SceneStaging {
    pub buffer: Buffer,
    pub regions: Vec<vk::BufferCopy>,
    pub bvh_offset: u32,
    pub bvh_len: u32,
}

#[derive(Clone, Copy)]
pub struct OptimalBufferCopyAlligment(usize);

impl SceneWorker {
    pub fn new_staging_builder(&mut self) -> SceneStagingBuilder {

        if let Some(buffer) = self.staging_buffers.pop() {
            SceneStagingBuilder {
                staging: SceneStaging {
                    buffer,
                    regions: vec![],
                    bvh_offset: self.bvh_allocation.start() as u32,
                    bvh_len: self.bvh_len as u32,
                },
                offset: 0,
                optimal_alignment: self.optimal_alignment,
            }
        } else {
            todo!("Alloc new staging buffer")
        }
    }
}

impl SceneStagingBuilder {
    pub fn update_bvh(&mut self, bvh_offset: u32, bvh_len: u32) {
        self.staging.bvh_offset = bvh_offset;
        self.staging.bvh_len = bvh_len;
    }

    pub fn push<T: Copy>(&mut self, data: &[T], gpu_offset: usize) {
        let data_size = size_of::<T>() * data.len();
        let size = if (data_size % self.optimal_alignment.0) == 0 {
            data_size
        } else {
            data_size + (self.optimal_alignment.0 - (data_size % self.optimal_alignment.0))
        };

        self.staging.buffer.copy_data_to_buffer_without_aligment(data, self.offset);
        self.staging.regions.push(vk::BufferCopy { 
            size: size as u64,
            src_offset: self.offset as u64,
            dst_offset: gpu_offset as u64, 
        });
        self.offset += size;
    }

    pub fn build(self) -> SceneStaging {
        self.staging
    }
}

impl OptimalBufferCopyAlligment {
    pub fn new(context: &Context) -> Self {
        Self(context.physical_device.limits.optimal_buffer_copy_offset_alignment
                .max(context.physical_device.limits.optimal_buffer_copy_row_pitch_alignment) as usize
        )
    }
}
