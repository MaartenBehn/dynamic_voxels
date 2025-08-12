use std::sync::{atomic::AtomicBool, Arc};

use bitvec::{array::BitArray, bitarr, order::Lsb0};
use octa_force::{anyhow::bail, glam::{vec3, vec4, Vec3A, Vec4}, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use parking_lot::{RwLock};

use super::{buffer::PaletteBuffer, palette::LocalPalette, Palette};

#[derive(Debug)]
pub struct SharedPalette {
    pub palette: Arc<RwLock<LocalPalette>>,
    pub changed: AtomicBool,
}

impl LocalPalette {
    pub fn shared(self) -> SharedPalette {
        SharedPalette { 
            palette: Arc::new(RwLock::new(self)),
            changed: AtomicBool::new(true),
        }
    }
}

impl Palette for SharedPalette {
    fn get_index_simple_color(&mut self, color: [u8; 3]) -> OctaResult<u8> {
        let mut palette = self.palette.upgradable_read();

        for i in palette.used.iter_ones().skip(1) {
            if palette.materials[i].is_simple_color() && palette.materials[i].color == color {
                return Ok(i as u8);
            }
        }

        palette.with_upgraded(|palette| {
            if let Some(i) = palette.used.first_zero() {
                palette.materials[i].set_simple_color(color);
                palette.used.set(i, true);
                return Ok(i as u8);
            } else {
                bail!("Palette full!");
            }
        })
    }
}



