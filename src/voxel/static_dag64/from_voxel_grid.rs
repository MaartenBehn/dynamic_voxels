use octa_force::glam::UVec3;

use crate::voxel::grid::VoxelGrid;

use super::StaticVoxelDAG64;

impl tree64::VoxelModel<u8> for VoxelGrid {
    fn dimensions(&self) -> [u32; 3] {
       self.size.into() 
    }

    fn access(&self, coord: [usize; 3]) -> Option<u8> {
        let pos = UVec3::new(coord[0] as u32, coord[1] as u32, coord[2] as u32);
        if pos.cmpge(self.size).any() {
            return None;
        }

        let data = self.get(pos);
        if (data == 0) {
            None
        } else {
            Some(data)
        }
    }
}

impl From<&VoxelGrid> for StaticVoxelDAG64 {
    fn from(value: &VoxelGrid) -> Self {
        let tree = tree64::Tree64::new(value);

        Self {
            tree
        }
    }
}
