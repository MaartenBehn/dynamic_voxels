pub mod material;
pub mod palette;
pub mod shared;
pub mod buffer;

use octa_force::OctaResult;

pub trait Palette {
    fn get_index_simple_color(&mut self, color: [u8; 3]) -> OctaResult<u8>;
}
