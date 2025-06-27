mod from_vec_csg_tree;
mod impl_volume;

use std::usize;

use octa_force::glam::{uvec3, vec3, UVec3, Vec3};

use crate::{csg_renderer::color_controller::MATERIAL_NONE, util::to_1d};

const VOXELS_PER_U32: usize = 4;

#[derive(Clone, Debug)]
pub struct VoxelGrid {
    pub size: UVec3,
    pub data: Vec<u8>,
}

impl VoxelGrid {
    pub fn new(size: UVec3) -> Self {
        let data_length = size.element_product() as usize;
        VoxelGrid {
            size,
            data: vec![MATERIAL_NONE as u8; data_length],
        }
    }

    pub fn set_example_sphere(&mut self) {
        let center = self.size / 2;
        let radius = (self.size.x as f32 / 3.0);
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                for z in 0..self.size.z {
                    let pos = uvec3(x as u32, y as u32, z as u32);
                    let index = to_1d(pos, self.size);

                    let dist = (center.as_vec3() - pos.as_vec3()).length();

                    if dist < radius {
                        self.data[index] = 2;
                    } else {
                        self.data[index] = 0;
                    }
                }
            }
        }
    }

    pub fn set_corners(&mut self) {
        for x in [0, self.size.x - 1] {
            for y in [0, self.size.y - 1] {
                for z in [0, self.size.z - 1] {
                    let pos = uvec3(x as u32, y as u32, z as u32);
                    let index = to_1d(pos, self.size);
                    self.data[index] = 2;
                }  
            }  
        }  
    }

    pub fn get(&self, pos: UVec3) -> u8 {
        self.data[to_1d(pos, self.size)]
    }
}
