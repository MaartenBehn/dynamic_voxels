use octa_force::glam::{ivec3, uvec3, IVec3, UVec3};

pub fn to_1d(pos: UVec3, size: UVec3) -> usize {
    (pos.x * size.y * size.z + pos.y * size.z + pos.z) as usize
}

pub fn to_1d_i(pos: IVec3, size: IVec3) -> usize {
    (pos.x * size.y * size.z + pos.y * size.z + pos.z) as usize
}

pub fn to_3d(index: usize, size: UVec3) -> UVec3 {
    // Should get optimzed to divmod opperation
    let x = index as u32 / (size.x * size.y);
    let rem = index as u32 % (size.x * size.y);

    // Should get optimzed to divmod opperation
    let y = rem / size.x;
    let z = rem % size.x;
    uvec3(x, y, z)
}

pub fn to_3d_i(index: usize, size: IVec3) -> IVec3 {
    // Should get optimzed to divmod opperation
    let x = index as i32 / (size.x * size.y);
    let rem = index as i32 % (size.x * size.y);

    // Should get optimzed to divmod opperation
    let y = rem / size.x;
    let z = rem % size.x;
    ivec3(x, y, z)
}
