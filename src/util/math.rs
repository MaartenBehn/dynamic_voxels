use std::iter;

use octa_force::glam::{ivec3, uvec3, vec3a, IVec3, UVec3, Vec3A};
use rayon::iter::IntoParallelRefIterator;


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

pub fn to_3d_ivec3(index: usize, size: IVec3) -> IVec3 {
    // Should get optimzed to divmod opperation
    let x = index as i32 / (size.x * size.y);
    let rem = index as i32 % (size.x * size.y);

    // Should get optimzed to divmod opperation
    let y = rem / size.x;
    let z = rem % size.x;
    ivec3(x, y, z)
}

pub fn to_3d_ivec4(index: usize, size: IVec3) -> IVec3 {
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

pub fn vec3a_to_nalgebra_point(v: Vec3A) -> nalgebra::Point3<f32> {
    nalgebra::Point3::new(v.x, v.y, v.z)
}

pub fn nalgebra_point_to_vec3a(v: nalgebra::Point3<f32>) -> Vec3A {
    vec3a(v.x, v.y, v.y)
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


pub fn get_dag_node_children() -> [UVec3; 64] {
    [
        uvec3(0, 0, 0),
        uvec3(0, 0, 1),
        uvec3(0, 0, 2),
        uvec3(0, 0, 3),
        uvec3(0, 1, 0),
        uvec3(0, 1, 1),
        uvec3(0, 1, 2),
        uvec3(0, 1, 3),
        uvec3(0, 2, 0),
        uvec3(0, 2, 1),
        uvec3(0, 2, 2),
        uvec3(0, 2, 3),
        uvec3(0, 3, 0),
        uvec3(0, 3, 1),
        uvec3(0, 3, 2),
        uvec3(0, 3, 3),
        uvec3(1, 0, 0),
        uvec3(1, 0, 1),
        uvec3(1, 0, 2),
        uvec3(1, 0, 3),
        uvec3(1, 1, 0),
        uvec3(1, 1, 1),
        uvec3(1, 1, 2),
        uvec3(1, 1, 3),
        uvec3(1, 2, 0),
        uvec3(1, 2, 1),
        uvec3(1, 2, 2),
        uvec3(1, 2, 3),
        uvec3(1, 3, 0),
        uvec3(1, 3, 1),
        uvec3(1, 3, 2),
        uvec3(1, 3, 3),
        uvec3(2, 0, 0),
        uvec3(2, 0, 1),
        uvec3(2, 0, 2),
        uvec3(2, 0, 3),
        uvec3(2, 1, 0),
        uvec3(2, 1, 1),
        uvec3(2, 1, 2),
        uvec3(2, 1, 3),
        uvec3(2, 2, 0),
        uvec3(2, 2, 1),
        uvec3(2, 2, 2),
        uvec3(2, 2, 3),
        uvec3(2, 3, 0),
        uvec3(2, 3, 1),
        uvec3(2, 3, 2),
        uvec3(2, 3, 3),
        uvec3(3, 0, 0),
        uvec3(3, 0, 1),
        uvec3(3, 0, 2),
        uvec3(3, 0, 3),
        uvec3(3, 1, 0),
        uvec3(3, 1, 1),
        uvec3(3, 1, 2),
        uvec3(3, 1, 3),
        uvec3(3, 2, 0),
        uvec3(3, 2, 1),
        uvec3(3, 2, 2),
        uvec3(3, 2, 3),
        uvec3(3, 3, 0),
        uvec3(3, 3, 1),
        uvec3(3, 3, 2),
        uvec3(3, 3, 3),
    ]
}

pub fn get_dag_node_children_xzy_i() -> [IVec3; 64] {
    [
        ivec3(0, 0, 0),
        ivec3(0, 1, 0),
        ivec3(0, 2, 0),
        ivec3(0, 3, 0),
        ivec3(0, 0, 1),
        ivec3(0, 1, 1),
        ivec3(0, 2, 1),
        ivec3(0, 3, 1),
        ivec3(0, 0, 2),
        ivec3(0, 1, 2),
        ivec3(0, 2, 2),
        ivec3(0, 3, 2),
        ivec3(0, 0, 3),
        ivec3(0, 1, 3),
        ivec3(0, 2, 3),
        ivec3(0, 3, 3),
        ivec3(1, 0, 0),
        ivec3(1, 1, 0),
        ivec3(1, 2, 0),
        ivec3(1, 3, 0),
        ivec3(1, 0, 1),
        ivec3(1, 1, 1),
        ivec3(1, 2, 1),
        ivec3(1, 3, 1),
        ivec3(1, 0, 2),
        ivec3(1, 1, 2),
        ivec3(1, 2, 2),
        ivec3(1, 3, 2),
        ivec3(1, 0, 3),
        ivec3(1, 1, 3),
        ivec3(1, 2, 3),
        ivec3(1, 3, 3),
        ivec3(2, 0, 0),
        ivec3(2, 1, 0),
        ivec3(2, 2, 0),
        ivec3(2, 3, 0),
        ivec3(2, 0, 1),
        ivec3(2, 1, 1),
        ivec3(2, 2, 1),
        ivec3(2, 3, 1),
        ivec3(2, 0, 2),
        ivec3(2, 1, 2),
        ivec3(2, 2, 2),
        ivec3(2, 3, 2),
        ivec3(2, 0, 3),
        ivec3(2, 1, 3),
        ivec3(2, 2, 3),
        ivec3(2, 3, 3),
        ivec3(3, 0, 0),
        ivec3(3, 1, 0),
        ivec3(3, 2, 0),
        ivec3(3, 3, 0),
        ivec3(3, 0, 1),
        ivec3(3, 1, 1),
        ivec3(3, 2, 1),
        ivec3(3, 3, 1),
        ivec3(3, 0, 2),
        ivec3(3, 1, 2),
        ivec3(3, 2, 2),
        ivec3(3, 3, 2),
        ivec3(3, 0, 3),
        ivec3(3, 1, 3),
        ivec3(3, 2, 3),
        ivec3(3, 3, 3),
    ]
}
