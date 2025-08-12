mod from_csg_tree;
mod impl_volume;
pub mod offset;
pub mod shared;

use std::{fmt, usize};

use octa_force::glam::{uvec3, vec3, UVec3, Vec3, Vec3A};

use crate::{util::math::to_1d, volume::VolumeBoundsI};

use super::palette::palette::MATERIAL_ID_NONE;

const VOXELS_PER_U32: usize = 4;

#[derive(Clone)]
pub struct VoxelGrid {
    pub data: Vec<u8>,
    pub size: UVec3,
}

impl VoxelGrid {
    pub fn empty(size: UVec3) -> Self {
        let data_length = size.element_product() as usize;
        VoxelGrid {
            size,
            data: vec![MATERIAL_ID_NONE as u8; data_length],
        }
    }

    pub fn from_data(size: UVec3, data: Vec<u8>) -> Self {
        VoxelGrid {
            size,
            data,
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
        debug_assert!(pos.cmpge(UVec3::ZERO).all() && pos.cmplt(self.size).all(), "Grid access at {pos} out of bounds! Size: {}", self.size);
        self.data[to_1d(pos, self.size)]
    }
}

impl fmt::Debug for VoxelGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VoxelGrid")
            .field("data", &())
            .field("size", &self.size)
            .finish()
    }
}
