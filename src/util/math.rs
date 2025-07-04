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

pub fn count_ones_variable(value: u64, index: u32) -> u32 {
    (value & ((1 << index) - 1)).count_ones()
}

pub fn set_bit(mask: u64, index: u32, value: bool) -> u64 {
    (mask & !(1 << index)) | ((value as u64) << index)
}

pub fn to_mb(size: usize) -> f32 {
    size as f32 * 0.000001
}

pub fn to_kb(size: usize) -> f32 {
    size as f32 * 0.001
}

/*
pub fn sphere_aabb_intersection(aabb: AABB)
{
    // Compute squared distance between sphere center and AABB
    // the sqrt(dist) is fine to use as well, but this is faster.
    float sqDist = SqDistPointAABB( s.center, b );

    // Sphere and AABB intersect if the (squared) distance between them is
    // less than the (squared) sphere radius.
    return sqDist <= s.r * s.r;
}

// Returns the squared distance between a point p and an AABB b
float SqDistPointAABB( Point p, AABB b )
{
    float sqDist = 0.0f;
    for( int i = 0; i < 3; i++ ){
        // for each axis count any excess distance outside box extents
        float v = p[i];
        if( v < b.min[i] ) sqDist += (b.min[i] - v) * (b.min[i] - v);
        if( v > b.max[i] ) sqDist += (v - b.max[i]) * (v - b.max[i]);
    }
    return sqDist;
}
*/
