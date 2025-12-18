pub mod material;
pub mod palette;
pub mod shared;
pub mod buffer;
pub mod picker;

use octa_force::OctaResult;

pub trait Palette {
    fn get_index_simple_color(&mut self, color: [u8; 3]) -> OctaResult<u8>;
    fn get_color(&self, mat: u8) -> [u8; 3];
    fn colors(&self) -> Vec<(u8, [u8; 3])>;
}
