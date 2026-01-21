use std::marker::PhantomData;

use isosurface::{marching_cubes::MarchingCubes};
use octa_force::glam::{Vec3, vec3a};

use crate::{mesh::{Mesh, Vertex}, util::{number::Nu, vector::Ve}, volume::VolumeQureyPosValid};


struct Source<'a, Vol: VolumeQureyPosValid<V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> {
    pub inner: &'a Vol,
    pub size: f32,
    p_1: PhantomData<V>, 
    p_2: PhantomData<T>, 
}

impl<Vol: VolumeQureyPosValid<V, T, 3>, V: Ve<T, 3>, T: Nu> isosurface::source::Source 
    for Source<'_, Vol, V, T, 3> {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
       
        let v = vec3a(x, y, z) * self.size;

        if self.inner.is_position_valid(V::from_vec3a(v)) { 1.0 } else { -1.0 }
    }
}

impl Mesh {
    pub fn from_volume<Vol: VolumeQureyPosValid<V, T, 3>, V: Ve<T, 3>, T: Nu>(vol: &Vol) -> Mesh {
        
        let size = vol.get_size().max_value().1;
        let s = Source {
            inner: vol,
            size: size.to_f32(),
            p_1: PhantomData,
            p_2: PhantomData,
        };

        let mut marching_cubes = MarchingCubes::new(size.to_usize());
       
        let mut vertices = vec![];
        let mut indices = vec![];
        marching_cubes.extract(&s, &mut vertices, &mut indices);

        let vertices = vertices.chunks(3)
            .map(|v| {
                Vertex { position: Vec3::new(v[0], v[1], v[2]) }
            })
            .collect();

        let aabb = vol.get_bounds();
        Mesh {
            vertices,
            indices,
            aabb: aabb.to_f3d(),
        }
    }
}
