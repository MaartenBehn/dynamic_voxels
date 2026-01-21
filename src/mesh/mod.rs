use isosurface::marching_cubes::MarchingCubes;
use octa_force::{glam::Vec3, vulkan::ash::vk};

use crate::{util::{aabb::{AABB, AABB3}, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

pub mod from_volume;
pub mod renderer;

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    aabb: AABB3,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct Vertex {
    position: Vec3,
}

impl octa_force::vulkan::Vertex for Vertex {
    fn bindings() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: 12,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attributes() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
        ]
    }
}

impl<V: Ve<f32, 3>> VolumeBounds<V, f32, 3> for Mesh {
    fn calculate_bounds(&mut self) {
        
    }

    fn get_bounds(&self) -> AABB<V, f32, 3> {
        AABB::new(
            V::from_vec3a(self.aabb.min()), 
            V::from_vec3a(self.aabb.max()))
    }
}



