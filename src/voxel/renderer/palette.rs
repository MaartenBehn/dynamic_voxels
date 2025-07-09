use octa_force::{glam::{vec3, vec4, Vec3, Vec4}, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};

pub const MATERIAL_ID_NONE: u8 = 0; 
pub const MATERIAL_ID_BASE: u8 = 1; 

#[derive(Debug)]
pub struct Palette {
    pub materials: [Material; 256],
    pub buffer: Buffer,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Material {
    color: [u8; 3],
    metal_fuzziness: u8,
    emission: half::f16,
}

impl Palette {
    pub fn new(context: &Context) -> OctaResult<Self> {
        let mut materials = [Material::default(); 256];
        materials[1].set_color(vec3(1.0, 1.0, 1.0));
        materials[2].set_color(vec3(0.5, 0.5, 0.5));
        materials[3].set_color(vec3(1.0, 1.0, 0.5));

        let buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::TRANSFER_DST, 
            MemoryLocation::GpuOnly,
            (size_of::<u64>() * 256) as _)?;

        let palette = Self { 
            materials,
            buffer 
        };

        palette.push_materials(context)?;

        Ok(palette)
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

    pub fn set_color(&mut self, color: Vec3) {
        let value = (color * 255.0).clamp(Vec3::ZERO, Vec3::ONE * 255.0);
        self.color[0] = value.x.round() as u8;
        self.color[1] = value.y.round() as u8;
        self.color[2] = value.z.round() as u8;
    }
}
