use octa_force::glam::Vec3;

use crate::voxel_grid::VoxelGrid;

pub mod from_voxel_grid;
pub mod renderer;

#[derive(Debug)]
pub struct VoxelTree64 {
    pub tree: tree64::Tree64<u8>    
}

impl VoxelTree64 {
    pub fn get_size(&self) -> Vec3 {
        let size = self.get_size_u32();
        Vec3::splat(size as f32)
    }

    pub fn get_size_u32(&self) -> u32 {
        4_u32.pow(self.tree.root_state().num_levels as u32 - 1)
    }


    pub fn get_root_index(&self) -> u32 {
        self.tree.root_state().index    
    }
}
