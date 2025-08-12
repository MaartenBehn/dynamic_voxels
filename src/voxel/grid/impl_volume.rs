use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A};

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI, math::to_1d}, volume::{VolumeBounds, VolumeBoundsI, VolumeQureyPosValue, VolumeQureyPosValueI}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid, VoxelGrid};

impl VolumeBoundsI for VoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds_i(&self) -> AABBI {
        AABBI::new(IVec3::ZERO, self.size.as_ivec3() -1)
    }

    fn get_size_i(&self) -> IVec3 {
        self.size.as_ivec3()
    }

    fn get_offset_i(&self) -> IVec3 {
        IVec3::ZERO
    }
}

impl VolumeBoundsI for OffsetVoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds_i(&self) -> AABBI {
        AABBI::new(self.offset, self.grid.size.as_ivec3() + self.offset - 1)
    }

    fn get_size_i(&self) -> IVec3 {
        self.grid.size.as_ivec3()
    }

    fn get_offset_i(&self) -> IVec3 {
        self.offset
    }
}

impl VolumeBoundsI for SharedVoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds_i(&self) -> AABBI {
        AABBI::new(self.offset, self.grid.size.as_ivec3() + self.offset -1)
    }

    fn get_size_i(&self) -> IVec3 {
        self.grid.size.as_ivec3()
    }

    fn get_offset_i(&self) -> IVec3 {
        self.offset
    }
}

impl VolumeBounds for VoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB {
        self.get_bounds_i().into()
    }
}

impl VolumeBounds for OffsetVoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB {
        self.get_bounds_i().into()
    }
}

impl VolumeBounds for SharedVoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB {
        self.get_bounds_i().into()
    }
}

impl VolumeQureyPosValueI for VoxelGrid {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        if pos.cmplt(IVec3::ZERO).any() || pos.cmpge(self.size.as_ivec3()).any() {
            return MATERIAL_ID_NONE;
        } 

        self.get(pos.as_uvec3())
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        self.get(pos)
    }
}

impl VolumeQureyPosValueI for OffsetVoxelGrid {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        let pos = pos - self.offset;
        self.grid.get_value_i(pos)
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        self.grid.get_value_relative_u(pos)
    }
}

impl VolumeQureyPosValueI for SharedVoxelGrid {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        let pos = pos - self.offset;
        self.grid.get_value_i(pos)
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        self.grid.get_value_relative_u(pos)
    }
}

impl VolumeQureyPosValue for VoxelGrid {
    fn get_value(&self, pos: Vec3A) -> u8 {
        self.get(pos.as_uvec3())
    }
}

impl VolumeQureyPosValue for OffsetVoxelGrid {
    fn get_value(&self, pos: Vec3A) -> u8 {
        self.get_value_i(pos.as_ivec3())
    }
}

impl VolumeQureyPosValue for SharedVoxelGrid {
    fn get_value(&self, pos: Vec3A) -> u8 {
        self.get_value_i(pos.as_ivec3())
    }
}
