use octa_force::glam::UVec3;

pub fn to_1d(pos: UVec3, size: UVec3) -> usize {
    (pos.x * size.y * size.z + pos.y * size.z + pos.z) as usize
}