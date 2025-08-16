use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A};

use crate::{util::{aabb::AABB, math::to_1d, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid, VoxelGrid};

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for VoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<V, T, D> {
        AABB::new(V::ZERO, V::from_uvec3(self.size -1))
    }

    fn get_size(&self) -> V {
        V::from_uvec3(self.size -1)
    }

    fn get_offset(&self) -> V {
        V::ZERO
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for OffsetVoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<V, T, D> {
        AABB::new(V::from_ivec3(self.offset), V::from_ivec3(self.grid.size.as_ivec3() + self.offset - 1))
    }

    fn get_size(&self) -> V {
        V::from_uvec3(self.grid.size)
    }

    fn get_offset(&self) -> V {
        V::from_ivec3(self.offset)
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for SharedVoxelGrid {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB<V, T, D> {
        AABB::new(V::from_ivec3(self.offset), V::from_ivec3(self.grid.size.as_ivec3() + self.offset - 1))
    }

    fn get_size(&self) -> V {
        V::from_uvec3(self.grid.size)
    }

    fn get_offset(&self) -> V {
         V::from_ivec3(self.offset)
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValue<V, T, D> for VoxelGrid {
    fn get_value(&self, pos: V) -> u8 {
        if pos.cmplt_any(V::ZERO) || pos.cmpge_any(V::from_uvec3(self.size)) {
            return MATERIAL_ID_NONE;
        } 

        self.get(pos.to_uvec3())
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValue<V, T, D> for OffsetVoxelGrid {
    fn get_value(&self, pos: V) -> u8 {
        let pos = pos - V::from_ivec3(self.offset);

        self.grid.get_value(pos)
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValue<V, T, D> for SharedVoxelGrid {
    fn get_value(&self, pos: V) -> u8 {
        let pos = pos - V::from_ivec3(self.offset);

        self.grid.get_value(pos)
    }
}
