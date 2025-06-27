use octa_force::glam::UVec3;

use crate::{util::to_1d, volume::VolumeQureyPosValue};

use super::VoxelGrid;


impl VolumeQureyPosValue for VoxelGrid {
    fn get_value(&self, pos: UVec3) -> u8 {
        if pos.cmpge(self.size).any() {
            return 0;
        }

        self.data[to_1d(pos, self.size)]
    }

    fn get_size(&self) -> UVec3 {
        self.size
    }
}
