use octa_force::glam::{IVec3, Vec3A};
use crate::{util::{math_config::MC, vector::Ve}, volume::{VolumeBounds, VolumeQureyAABB}};

pub fn get_dag_offset_levels<C: MC<3>, M: VolumeBounds<C, 3>>(model: &M) -> (IVec3, u8) {
    let offset = model.get_offset().to_ivec3();
    let dims = model.get_size().to_uvec3();
    let mut scale = dims[0].max(dims[1]).max(dims[2]).next_power_of_two();
    scale = scale.max(4);
    if scale.ilog2() % 2 == 1 {
        scale *= 2;
    }

    let levels = scale.ilog(4) as _;

    (offset, levels)
}




