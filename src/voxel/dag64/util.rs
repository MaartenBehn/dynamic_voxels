use octa_force::{glam::{IVec3, UVec3, Vec3A}, OctaResult};
use crate::{util::{math_config::MC, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyAABB}, voxel::{dag64::lod_heuristic::LODHeuristicT, grid::offset}};

pub fn get_dag_offset_levels<V: Ve<T, 3>, T: Nu, M: VolumeBounds<V, T, 3>>(model: &M) -> (IVec3, u8) {
    let dims: UVec3 = model.get_size().ve_into();
    if dims == UVec3::ZERO {
        return (IVec3::ZERO, 0);
    }

    let mut size = dims[0].max(dims[1]).max(dims[2]).next_power_of_two();
    size = size.max(4);
    if size.ilog2() % 2 == 1 {
        size *= 2;
    }

    let levels = size.ilog(4) as _;

    let aabb = model.get_bounds();
    let aabb_offset: IVec3 = aabb.min().ve_into();
    let aabb_size: IVec3 = aabb.size().ve_into();

    let offset = aabb_offset + (aabb_size - IVec3::splat(size as i32)) / 2;
    
    (offset, levels)
}

pub fn get_voxel_size(level: u8) -> i32 {
    1 << (2 * level)
}


