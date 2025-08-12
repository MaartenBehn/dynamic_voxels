
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Material {
    pub color: [u8; 3],
    pub metal_fuzziness: u8,
    pub emission: half::f16,
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
