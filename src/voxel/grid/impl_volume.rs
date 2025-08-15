use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A};

use crate::{util::{aabb::AABB, aabb3d::AABB3, iaabb3d::AABBI, math::to_1d, math_config::MC, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid, VoxelGrid};

impl<C: MC<3>> VolumeBounds<C, 3> for VoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<C, 3> {
        AABB::new(C::Vector::ZERO, C::Vector::from_uvec3(self.size -1))
    }

    fn get_size(&self) -> C::Vector {
        C::Vector::from_uvec3(self.size -1)
    }

    fn get_offset(&self) -> C::Vector {
        C::Vector::ZERO
    }
}

impl<C: MC<3>> VolumeBounds<C, 3> for OffsetVoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<C, 3> {
        AABB::new(C::Vector::from_ivec3(self.offset), C::Vector::from_ivec3(self.grid.size.as_ivec3() + self.offset - 1))
    }

    fn get_size(&self) -> C::Vector {
        C::Vector::from_uvec3(self.grid.size)
    }

    fn get_offset(&self) -> C::Vector {
        C::Vector::from_ivec3(self.grid.size)
    }
}

impl<C: MC<3>> VolumeBounds<C, 3> for SharedVoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<C, 3> {
        AABB::new(C::Vector::from_ivec3(self.offset), C::Vector::from_ivec3(self.grid.size.as_ivec3() + self.offset - 1))
    }

    fn get_size(&self) -> C::Vector {
        C::Vector::from_uvec3(self.grid.size)
    }

    fn get_offset(&self) -> C::Vector {
        C::Vector::from_uvec3(self.grid.size)
    }
}

impl<C: MC<3>> VolumeQureyPosValue<C, 3> for VoxelGrid {
    fn get_value(&self, pos: C::Vector) -> u8 {
        if pos.cmplt_any(C::Vector::ZERO) || pos.cmpge_any(C::Vector::from_uvec3(self.size)) {
            return MATERIAL_ID_NONE;
        } 

        self.get(pos.to_uvec3())
    }
}

impl<C: MC<3>> VolumeQureyPosValue<C, 3> for OffsetVoxelGrid {
    fn get_value(&self, pos: C::Vector) -> u8 {
        let pos = pos - C::Vector::from_ivec3(self.offset);
        self.grid.get_value(pos)
    }
}

impl<C: MC<3>> VolumeQureyPosValue<C, 3> for SharedVoxelGrid {
    fn get_value(&self, pos: C::Vector) -> u8 {
        let pos = pos - C::Vector::from_ivec3(self.offset);
        self.grid.get_value(pos)
    }
}
