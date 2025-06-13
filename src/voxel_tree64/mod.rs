use crate::voxel_grid::VoxelGrid;

pub mod from_voxel_grid;
pub mod renderer;

pub struct VoxelTree64 {
    pub tree: tree64::Tree64<u8>    
}

