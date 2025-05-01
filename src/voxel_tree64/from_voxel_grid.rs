use octa_force::glam::UVec3;

use crate::{util::to_1d, voxel_grid::VoxelGrid};

use super::VoxelTree64;

impl tree64::VoxelModel<u8> for VoxelGrid {
    fn dimensions(&self) -> [u32; 3] {
       self.size.into() 
    }

    fn access(&self, coord: [usize; 3]) -> Option<u8> {
        let pos = UVec3::new(coord[0] as u32, coord[1] as u32, coord[2] as u32);
        if pos.cmpge(self.size).any() {
            return None;
        }

        Some(self.data[to_1d(pos, self.size)])
    }
}

impl From<VoxelGrid> for VoxelTree64 {
    fn from(value: VoxelGrid) -> Self {
        let tree = tree64::Tree64::new(value);

        Self {
            tree
        }
    }
}
