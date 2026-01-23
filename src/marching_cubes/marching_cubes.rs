use octa_force::glam::Vec3;

use crate::METERS_PER_SHADER_UNIT;
use crate::mesh::Vertex;
use crate::util::aabb::{AABB, AABB3};
use crate::util::number::Nu;
use crate::util::vector::Ve;
use crate::volume::VolumeQureyPosValue;
use crate::voxel::palette::palette::MATERIAL_ID_NONE;

use super::index_cache::IndexCache;
use super::marching_cubes_impl::{get_offset, interpolate, march_cube};
use super::marching_cubes_tables::{CORNERS, EDGE_CONNECTION};

pub fn marching_cubes<S: VolumeQureyPosValue<V, f32, 3>, V: Ve<f32, 3>>(
    source: &S,
    vertices: &mut Vec<Vertex>, 
    indices: &mut Vec<u32>
) {

    let aabb = source.get_bounds();
    let min = aabb.min().to_ivec3();
    let max = aabb.max().to_ivec3();
    let size = (max - min).as_uvec3() + 1;
    let size_x = size.x as usize;

    let mut layers = [vec![MATERIAL_ID_NONE; (size.x * size.y) as usize], vec![MATERIAL_ID_NONE; (size.x * size.y) as usize]];

    // Cache layer zero of distance field values
    for (j, y) in (min.y..=max.y).into_iter().enumerate() {
        for (i, x) in (min.x..=max.x).into_iter().enumerate() {
            layers[0][j * size_x + i] =
                source.get_value(V::new([x as f32, y as f32, min.z as f32]));
        }
    }

    let mut corners = [Vec3::ZERO; 8];
    let mut values = [MATERIAL_ID_NONE; 8];

    let mut index_cache = IndexCache::new(size);
    let mut index = 0u32;

    for (k, z) in (min.z..=max.z).into_iter().enumerate() {
        // Cache layer N+1 of isosurface values
        for (j, y) in (min.y..=max.y).into_iter().enumerate() {
            for (i, x) in (min.x..=max.x).into_iter().enumerate() {
                layers[1][j * size_x + i] = source.get_value(
                    V::new([
                        x as f32,
                        y as f32,
                        (z + 1) as f32])
                );
            }
        }

        // Extract the calls in the current layer
        for (j, y) in (min.y..max.y).into_iter().enumerate() {
            for (i, x) in (min.x..max.x).into_iter().enumerate() {
                for l in 0..8 {
                    corners[l] = Vec3::new(
                        (x + CORNERS[l][0] as i32) as f32,
                        (y + CORNERS[l][1] as i32) as f32,
                        (z + CORNERS[l][2] as i32) as f32,
                    );
                    values[l] = layers[CORNERS[l][2]]
                        [(j + CORNERS[l][1]) * size_x + i + CORNERS[l][0]];
                }

                march_cube(&values, |edge: usize| {
                    let cached_index = index_cache.get(i, j, edge);
                    if cached_index > 0 {
                        indices.push(cached_index);
                    } else {
                        let u = EDGE_CONNECTION[edge][0];
                        let v = EDGE_CONNECTION[edge][1];

                        index_cache.put(i, j, edge, index);
                        indices.push(index);
                        index += 1;

                        //let offset = get_offset(values[u], values[v]);
                        //let vertex = interpolate(corners[u], corners[v], offset);

                        let vertex = (corners[u] + corners[v]) * 0.5;

                        let pos = vertex; 
                        let value = if values[u] != MATERIAL_ID_NONE {
                            values[u]
                        } else if values[v] != MATERIAL_ID_NONE {
                            values[v]
                        } else {
                            MATERIAL_ID_NONE
                        };

                        vertices.push(Vertex::new(pos / METERS_PER_SHADER_UNIT as f32, value));
                    }
                });
                index_cache.advance_cell();
            }
            index_cache.advance_row();
        }
        index_cache.advance_layer();

        layers.swap(0, 1);
    }
}

