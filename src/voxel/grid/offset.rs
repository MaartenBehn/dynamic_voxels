use octa_force::glam::{IVec3, UVec3};

use super::{shared::SharedVoxelGrid, VoxelGrid};


#[derive(Clone, Debug)]
pub struct OffsetVoxelGrid {
    pub grid: VoxelGrid,
    pub offset: IVec3,
}

impl OffsetVoxelGrid {
    pub fn empty(size: UVec3, offset: IVec3) -> Self {
        Self {
            grid: VoxelGrid::empty(size),
            offset,
        }
    }

    pub fn from_data(size: UVec3, data: Vec<u8>, offset: IVec3) -> Self {
        Self {
            grid: VoxelGrid::from_data(size, data),
            offset,
        }
    }

    pub fn from_grid(grid: VoxelGrid, offset: IVec3) -> Self {
        Self {
            grid,
            offset,
        }
    }
}

impl Into<SharedVoxelGrid> for OffsetVoxelGrid {
    fn into(self) -> SharedVoxelGrid {
        SharedVoxelGrid::from_grid(self.grid, self.offset)
    }
}
