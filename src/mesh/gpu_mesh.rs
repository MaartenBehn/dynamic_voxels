use octa_force::{glam::vec3, vulkan::{Buffer, Context, ash::vk::BufferUsageFlags}};

use crate::{METERS_PER_SHADER_UNIT, mesh::{Mesh, Vertex}};

#[derive(Debug)]
pub struct GPUMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: usize,
}

impl Mesh {
    pub fn flush_to_gpu(&self, contex: &Context) -> GPUMesh {
        let vertex_buffer = contex.create_gpu_only_buffer_from_data(
            BufferUsageFlags::VERTEX_BUFFER,
            &self.vertices)
            .expect("Failed to push vertex buffer");

        let index_buffer = contex.create_gpu_only_buffer_from_data(
            BufferUsageFlags::INDEX_BUFFER,
            &self.indices)
            .expect("Failed to push index buffer");

        GPUMesh {
            vertex_buffer,
            index_buffer,
            index_count: self.indices.len(),
        }
    }
}
