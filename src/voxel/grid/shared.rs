use std::sync::Arc;
use octa_force::glam::{IVec3, UVec3};
use parking_lot::Mutex;

use super::{offset::OffsetVoxelGrid, VoxelGrid};

#[derive(Clone, Debug)]
pub struct SharedVoxelGrid {
    pub grid: Arc<Mutex<VoxelGrid>>,
    pub offset: IVec3,
}

impl SharedVoxelGrid {
    pub fn empty(size: UVec3, offset: IVec3) -> Self {
        Self {
            grid: Arc::new(Mutex::new(VoxelGrid::empty(size))),
            offset,
        }
    }

    pub fn from_data(size: UVec3, data: Vec<u8>, offset: IVec3) -> Self {
        Self {
            grid: Arc::new(Mutex::new(VoxelGrid::from_data(size, data))),
            offset,
        }
    }

    pub fn from_grid(grid: VoxelGrid, offset: IVec3) -> Self {
        Self {
            grid: Arc::new(Mutex::new(grid)),
            offset,
        }
    }
}

impl Into<OffsetVoxelGrid> for SharedVoxelGrid {
    fn into(self) -> OffsetVoxelGrid {
        let grid = Arc::try_unwrap(self.grid).unwrap().into_inner();
        OffsetVoxelGrid::from_grid(grid, self.offset)
    }
}


