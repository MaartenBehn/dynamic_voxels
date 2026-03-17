use std::sync::Arc;

use octa_force::glam::{IVec3, UVec3, Vec3, uvec3};

use crate::{util::{math::get_dag_node_children_xzy_i, parallel_pool::ParallelPool, vector::Ve}, voxel::{dag64::{node::VoxelDAG64Node, util::get_voxel_size}, renderer::g_buffer::ImageAndViewAndHandle}};

pub const GI_PROBE_INDEX_NONE: u32 = u32::MAX;
pub const GI_PROBE_MIN_LEVEL: u8 = 2;

#[derive(Debug)]
pub struct GIPoolAtlas {
    pub images: Vec<ImageAndViewAndHandle>,
}

#[derive(Debug)]
pub struct GIPool {
    pub pools: Vec<ParallelPool<GIProbe>>,
    search_order: [(usize, IVec3); 64],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GIProbe {
    pub position: IVec3,
}

impl GIPool {
    pub fn new(levels: usize) -> Self {
        
        let mut pools = vec![];

        for level in (0..levels).rev() {
            let size = get_voxel_size(level as _);
            pools.push(ParallelPool::new(size as _));
        }

        Self {
            pools,
            search_order: search_order(),
        }
    }

    pub fn new_probe_index(&self, offset: IVec3, level: u8, pop_mask: u64, children: &[VoxelDAG64Node]) -> u32 {
        if level < GI_PROBE_MIN_LEVEL {
            return GI_PROBE_INDEX_NONE;
        }

        let pos = self.find_position(offset, level, pop_mask, children);
        if pos.is_none() {
            return GI_PROBE_INDEX_NONE;
        }

        let gi_level = &self.pools[level as usize];
        gi_level.insert(GIProbe {
            position: pos.unwrap(),
        }).expect(&format!("Probe pool full at level: {level}")) as u32
    }

    fn find_position(&self, offset: IVec3, level: u8, pop_mask: u64, children: &[VoxelDAG64Node]) -> Option<IVec3> {

        if pop_mask != u64::MAX {
            let child_size = get_voxel_size(level - 1);
            for (i, pos) in self.search_order {
                if pop_mask >> i & 1 == 0 {
                    return Some(offset + pos * child_size);
                } 
            }
            unreachable!();
        }

        // No empty children we search one level down
        let child_size = get_voxel_size(level - 2);
        for (i, pos) in search_order() {
            let child = children[i];

            if child.pop_mask == u64::MAX {
                continue;
            }

            for (i, pos) in self.search_order {
                if child.pop_mask >> i & 1 == 0 {
                    return Some(offset + pos * child_size);
                } 
            }
            unreachable!();
        }

        None
    }
}


fn search_order() -> [(usize, IVec3); 64] {

    let mut positions = Vec::new();
    let center = Vec3::splat(1.5);
    for (i, pos) in get_dag_node_children_xzy_i().into_iter().enumerate() {
        let dist = pos.as_vec3().distance(center);

        positions.push((dist, pos, i));
    }

    positions.sort_by(|a, b| a.0.total_cmp(&b.0)); 
    let positions: Vec<_> = positions.into_iter().map(|a| (a.2, a.1)).collect();
    positions.try_into().unwrap()
}

