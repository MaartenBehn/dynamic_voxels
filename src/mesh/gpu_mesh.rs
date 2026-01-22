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

        let offset = vec3(0.0,0.0,0.0);   
        let mut vertices = vec![
            Vertex::new(vec3(-0.5,  0.5, -0.5) + offset),
            Vertex::new(vec3( 0.5,  0.5, -0.5) + offset),
            Vertex::new(vec3( 0.5,  0.5,  0.5) + offset),
            Vertex::new(vec3(-0.5,  0.5,  0.5) + offset),
            Vertex::new(vec3(-0.5, -0.5, -0.5) + offset),
            Vertex::new(vec3( 0.5, -0.5, -0.5) + offset),
            Vertex::new(vec3( 0.5, -0.5,  0.5) + offset),
            Vertex::new(vec3(-0.5, -0.5,  0.5) + offset),
            Vertex::new(vec3(-1.0,  0.0,  0.0) + offset),
            Vertex::new(vec3( 1.0,  0.0,  0.0) + offset),
            Vertex::new(vec3( 0.0,  1.0,  0.0) + offset),
            Vertex::new(vec3( 0.0, -1.0,  0.0) + offset),
            Vertex::new(vec3( 0.0,  0.0,  1.0) + offset),
            Vertex::new(vec3( 0.0,  0.0, -1.0) + offset),
        ];

        for v in vertices.iter_mut() {
            v.pos /= METERS_PER_SHADER_UNIT as f32;
        }

        let indices = vec![
            3, 12, 2, 2, 10, 3, 
            2, 12, 6, 6, 9, 2, 
            6, 12, 7, 7, 11, 6, 
            7, 12, 3, 3, 8, 7, 
            4, 13, 5, 5, 11, 4,
            0, 13, 4, 4, 8, 0, 
            1, 13, 0, 0, 10, 1,
            5, 13, 1, 1, 9, 5,
            0, 8, 3, 3, 10, 0, 
            7, 8, 4, 4, 11, 7, 
            2, 9, 1, 1, 10, 2, 
            5, 9, 6, 6, 11, 5,
        ];

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
