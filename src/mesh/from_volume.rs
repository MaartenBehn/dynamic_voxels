use std::marker::PhantomData;

use octa_force::glam::{Vec3, Vec3A, vec3a};

use crate::{METERS_PER_SHADER_UNIT, marching_cubes::marching_cubes::marching_cubes, mesh::{Mesh, Vertex}, util::{number::Nu, vector::Ve}, volume::{VolumeQureyPosValid, VolumeQureyPosValue}, voxel::palette::{Palette, shared::SharedPalette}};


impl Mesh {
    pub fn from_volume<Vol, V: Ve<f32, 3>>(vol: &Vol) -> Mesh 
    where 
        Vol: VolumeQureyPosValue<V, f32, 3>,
    {
        
        let mut vertices = vec![];
        let mut indices = vec![];
        marching_cubes(vol, &mut vertices, &mut indices);

        let aabb = vol.get_bounds();
        Mesh {
            vertices,
            indices,
            aabb: aabb.to_f3d(),
        }
    }
}
