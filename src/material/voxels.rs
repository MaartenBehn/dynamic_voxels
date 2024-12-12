use crate::util::to_1d;
use octa_force::glam::{uvec3, UVec3, Vec3};

const VOXELS_PER_U32: usize = 4;

pub struct VoxelField {
    pub size: usize,
    pub buffer_size: usize,
    pub buffer_start: usize,
    pub data: Vec<u8>,
}

impl VoxelField {
    pub fn new(size: usize) -> Self {
        let data_size = size * size * size;
        VoxelField {
            size,
            buffer_size: (data_size / VOXELS_PER_U32) + 1,
            buffer_start: 0,
            data: vec![0; data_size],
        }
    }

    pub fn set_example_sphere(&mut self) {
        let center = Vec3::ONE * (self.size as f32 / 2.0);
        let radius = 3.0;
        for x in 0..self.size {
            for y in 0..self.size {
                for z in 0..self.size {
                    let pos = uvec3(x as u32, y as u32, z as u32);
                    let index = to_1d(pos, UVec3::ONE * self.size as u32);

                    let dist = (center - pos.as_vec3()).length();

                    if dist < radius {
                        self.data[index] = 2;
                    } else {
                        self.data[index] = 0;
                    }
                }
            }
        }
    }
}

