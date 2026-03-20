use octa_force::vulkan::{Buffer, Context, ash::vk};

use crate::scene::worker::SceneWorker;

pub struct SceneStagingBuilder {
    buffer: Buffer,
    regions: Vec<vk::BufferCopy>,
    offset: usize,
    optimal_alignment: OptimalBufferCopyAlligment,
    force_send: bool,
}

#[derive(Debug)]
pub struct SceneStaging {
    pub buffer: Buffer,
    pub regions: Vec<vk::BufferCopy>,
    pub bvh_offset: u32,
    pub bvh_len: u32,
    pub active_probe_map_offset: u32,
    pub active_probe_data_offset: u32,
    pub num_active_probes: u32,
}

#[derive(Clone, Copy)]
pub struct OptimalBufferCopyAlligment(usize);

impl SceneWorker {
    pub fn new_staging_builder(&mut self) -> SceneStagingBuilder {

        if let Some(buffer) = self.staging_buffers.pop() {
            SceneStagingBuilder {
                buffer,
                regions: vec![],
                offset: 0,
                optimal_alignment: self.optimal_alignment,
                force_send: false,
            }
        } else {
            todo!("Alloc new staging buffer")
        }
    }

    pub fn discard_builder(&mut self, builder: SceneStagingBuilder) {
        self.staging_buffers.push(builder.buffer);
    }

    pub fn build_builder(&mut self, builder: SceneStagingBuilder) -> SceneStaging {
        SceneStaging { 
            buffer: builder.buffer, 
            regions: builder.regions, 
            bvh_offset: self.bvh_allocation.start() as u32, 
            bvh_len: self.bvh_len as u32, 
            active_probe_map_offset: self.gi.active.probe_map_alloc.start() as u32, 
            active_probe_data_offset: self.gi.active.probe_data_alloc.start() as u32, 
            num_active_probes: self.gi.active.active_size,
        }
    }
}

impl SceneStagingBuilder {
    pub fn mark_send(&mut self) {
        self.force_send = true;
    }

    pub fn push<T: Copy>(&mut self, data: &[T], gpu_offset: usize) {
        if (data.is_empty()) {
            return;
        }

        let data_size = size_of::<T>() * data.len();
        let size = if (data_size % self.optimal_alignment.0) == 0 {
            data_size
        } else {
            data_size + (self.optimal_alignment.0 - (data_size % self.optimal_alignment.0))
        };

        self.buffer.copy_data_to_buffer_without_aligment(data, self.offset);
        self.regions.push(vk::BufferCopy { 
            size: size as u64,
            src_offset: self.offset as u64,
            dst_offset: gpu_offset as u64, 
        });
        self.offset += size;
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty() && !self.force_send
    }
}

impl OptimalBufferCopyAlligment {
    pub fn new(context: &Context) -> Self {
        Self(context.physical_device.limits.optimal_buffer_copy_offset_alignment
                .max(context.physical_device.limits.optimal_buffer_copy_row_pitch_alignment) as usize
        )
    }
}
