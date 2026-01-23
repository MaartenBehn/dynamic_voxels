use bitvec::{array::BitArray, bitarr, order::Lsb0};
use octa_force::{OctaResult, anyhow::bail, glam::{Vec3A, Vec4, uvec3, vec3, vec4}, vulkan::{Buffer, Context, ash::vk, gpu_allocator::MemoryLocation}};

use super::{material::Material, Palette};

pub const MATERIAL_ID_NONE: u8 = 0; 
pub const MATERIAL_ID_BASE: u8 = 1; 

#[derive(Debug)]
pub struct LocalPalette {
    pub materials: [Material; 256],
    pub used: BitArray<[u64; 4]>,
}

impl LocalPalette {
    pub fn new() -> Self {
        let mut materials = [Material::default(); 256];
        let mut used = bitarr![u64, Lsb0; 0; 256];
        
        used.set(0, true);
        materials[1].set_simple_color([255, 255, 255]);
        used.set(1, true);

        Self { 
            materials,
            used, 
        }
    }
}

impl Palette for LocalPalette {
    fn get_index_simple_color(&mut self, color: [u8; 3]) -> OctaResult<u8> {
        for i in self.used.iter_ones().skip(1) {
            if self.materials[i].is_simple_color() && self.materials[i].color == color {
                return Ok(i as u8);
            }
        }

        if let Some(i) = self.used.first_zero() {
            self.materials[i].set_simple_color(color);
            self.used.set(i, true);
            return Ok(i as u8);
        } else {
            bail!("Palette full!");
        }
    }

    fn colors(&self) -> Vec<(u8, [u8; 3])> {
        self.used.iter_ones()
            .skip(1)
            .map(|i| (i as u8, self.materials[i].color))
            .collect() 
    }

    fn get_color(&self, mat: u8) -> [u8; 3] {
        self.materials[mat as usize].color
    }
}

pub fn get_rgb_color(c: Vec3A) -> [u8; 3] {
    let value = (c * 255.0).clamp(Vec3A::ZERO, Vec3A::ONE * 255.0).round();
    [value.x as u8, value.y as u8, value.z as u8]
}

