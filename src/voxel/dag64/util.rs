use octa_force::glam::{IVec3, Vec3A};
use crate::volume::{VolumeBoundsI, VolumeQureyAABB, VolumeQureyPosValueI};

pub fn get_dag_offset_levels<M: VolumeBoundsI>(model: &M) -> (IVec3, u8) {
    let offset = model.get_offset_i();
    let dims = model.get_size_i().as_uvec3();
    let mut scale = dims[0].max(dims[1]).max(dims[2]).next_power_of_two();
    scale = scale.max(4);
    if scale.ilog2() % 2 == 1 {
        scale *= 2;
    }

    let levels = scale.ilog(4) as _;

    (offset, levels)
}




