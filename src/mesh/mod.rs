use isosurface::marching_cubes::MarchingCubes;

use crate::{util::{aabb::{AABB, AABB3}, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

pub mod from_volume;

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    vertices: Vec<f32>,
    indices: Vec<u32>,
    aabb: AABB3,
}

impl<V: Ve<f32, 3>> VolumeBounds<V, f32, 3> for Mesh {
    fn calculate_bounds(&mut self) {
        
    }

    fn get_bounds(&self) -> AABB<V, f32, 3> {
        AABB::new(
            V::from_vec3a(self.aabb.min()), 
            V::from_vec3a(self.aabb.max()))
    }
}



