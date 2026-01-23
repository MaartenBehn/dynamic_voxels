

use crate::{voxel::palette::palette::MATERIAL_ID_NONE};

use super::marching_cubes_tables::TRIANGLE_CONNECTION;
use std::ops::{Add, Mul};

/// March a single cube, given the 8 corner vertices, and the density at each vertex.
///
/// The `edge_func` will be invoked once for each vertex in the resulting mesh data, with the index
/// of the edge on which the vertex falls. Each triplet of invocations forms one triangle.
///
/// It would in many ways be simple to output triangles directly, but callers needing to produce
/// indexed geometry will want to deduplicate vertices before forming triangles.
pub fn march_cube<E>(values: &[u8; 8], mut edge_func: E)
where
    E: FnMut(usize) -> (),
{
    let mut cube_index = 0;
    for i in 0..8 {
        if values[i] == MATERIAL_ID_NONE {
            cube_index |= 1 << i;
        }
    }

    for i in 0..5 {
        if TRIANGLE_CONNECTION[cube_index][3 * i] < 0 {
            break;
        }

        for j in 0..3 {
            let edge = TRIANGLE_CONNECTION[cube_index][3 * i + j] as usize;

            edge_func(edge);
        }
    }
}

/// Calculate the position of the vertex along an edge, given the density at either end of the edge.
pub fn get_offset(a: f32, b: f32) -> f32 {
    let delta = b - a;
    if delta == 0.0 {
        0.5
    } else {
        -a / delta
    }
}

/// Linearly Interpolate between two floating point values
pub fn interpolate<T>(a: T, b: T, t: f32) -> T
where
    T: Add<T, Output = T> + Mul<f32, Output = T>,
{
    a * (1.0 - t) + b * t
}
