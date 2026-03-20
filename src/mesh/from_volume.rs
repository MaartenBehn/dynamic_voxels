use std::marker::PhantomData;

use octa_force::glam::{Vec3, Vec3A, Vec3Swizzles, vec3a};

use crate::{marching_cubes::marching_cubes::marching_cubes, mesh::{Mesh, Vertex}, util::{number::Nu, shader_constants::VOXELS_PER_SHADER_UNIT, vector::Ve}, volume::{VolumeGradient, VolumeQureyPosValid, VolumeQureyPosValue}, voxel::palette::{Palette, shared::SharedPalette}};


impl Mesh {
    pub fn from_volume<Vol, V: Ve<f32, 3>>(vol: &Vol) -> Mesh 
    where 
        Vol: VolumeQureyPosValue<V, f32, 3> + VolumeGradient<V, 3>,
    {
        
        let mut vertices = vec![];
        let mut indices = vec![];
        marching_cubes(vol, |pos, val| {
                let grad: Vec3 = vol.get_gradient_at_position(V::ve_from(pos)).normalize().ve_into();

                vertices.push(Vertex::new(
                pos.yxz() / VOXELS_PER_SHADER_UNIT as f32, 
                val, 
                grad));

            }, |i| {
                indices.push(i);
            });

        let indices = indices.chunks(3)
            .map(|t| { t.iter().rev() })
            .flatten()
            .copied()
            .collect();

        let aabb = vol.get_bounds();
        Mesh {
            vertices,
            indices,
            aabb: aabb.to_f3d(),
        }
    }
}
