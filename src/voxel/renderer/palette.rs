use bitvec::{array::BitArray, bitarr, order::Lsb0};
use octa_force::{anyhow::bail, glam::{vec3, vec4, Vec3A, Vec4}, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};

pub const MATERIAL_ID_NONE: u8 = 0; 
pub const MATERIAL_ID_BASE: u8 = 1; 

#[derive(Debug)]
pub struct Palette {
    pub materials: [Material; 256],
    pub used: BitArray<[u64; 4]>,
    pub buffer: Buffer,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Material {
    pub color: [u8; 3],
    pub metal_fuzziness: u8,
    pub emission: half::f16,
}

impl Palette {
    pub fn new(context: &Context) -> OctaResult<Self> {
        let mut materials = [Material::default(); 256];
        let mut used = bitarr![u64, Lsb0; 0; 256];
        
        used.set(0, true);
        materials[1].set_simple_color([255, 255, 255]);
        used.set(1, true);

        let buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::TRANSFER_DST, 
            MemoryLocation::GpuOnly,
            (size_of::<u64>() * 256) as _)?;

        let palette = Self { 
            materials,
            used, 
            buffer 
        };

        palette.push_materials(context)?;

        Ok(palette)
    }

    pub fn get_index_simple_color(&mut self, color: [u8; 3]) -> OctaResult<u8> {
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

    pub fn push_materials(&self, context: &Context) -> OctaResult<()> {
        let packed: Vec<_> = self.materials.iter().map(|m| m.get_encoded()).collect();

        // TODO Maybe reuse staging buffer?
        context.copy_data_to_gpu_only_buffer(&packed, &self.buffer)
    }
}

impl Material {
    pub fn get_encoded(&self) -> u64 {
        // Color (RGB565): u16
        // Emission:       f16
        // Fuzziness:      unorm8
        let mut packed = 0;

        packed |= (self.color[0] as u64) >> (8 - 5) << 11;
        packed |= (self.color[1] as u64) >> (8 - 6) << 5;
        packed |= (self.color[2] as u64) >> (8 - 5) << 0;
        packed |= (self.emission.to_bits() as u64) << 16;

        packed |= (self.metal_fuzziness as u64) << 32;

        return packed;
    }

    pub fn is_simple_color(&self) -> bool {
        self.emission == half::f16::ZERO && self.metal_fuzziness == 0
    } 

    pub fn set_simple_color(&mut self, rgb_color: [u8; 3]) {
        self.color = rgb_color;
        self.emission = half::f16::ZERO;
        self.metal_fuzziness = 0;
    }
}

pub fn get_rgb_color(c: Vec3A) -> [u8; 3] {
    let value = (c * 255.0).clamp(Vec3A::ZERO, Vec3A::ONE * 255.0).round();
    [value.x as u8, value.y as u8, value.z as u8]
}
