use std::sync::Arc;

use octa_force::glam::{IVec3, UVec3, Vec3, uvec3};

use crate::{util::{math::get_dag_node_children_i, vector::Ve}, voxel::{dag64::{node::VoxelDAG64Node, util::get_voxel_size}, renderer::g_buffer::ImageAndViewAndHandle}};

pub const GI_PROBE_INDEX_NONE: u32 = u32::MAX;
pub const GI_PROBE_MIN_LEVEL: u8 = 2;

#[derive(Debug)]
pub struct GIPoolAtlas {
    pub images: Vec<ImageAndViewAndHandle>,
}

#[derive(Debug)]
pub struct GIPool {
    pub pools: Vec<sharded_slab::Slab<GIProbe>>,
    search_order: [(usize, IVec3); 64],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GIProbe {
    pub position: IVec3,
    pub object_index: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct GIExecutor<'a> {
    pool: &'a GIPool,
    object_index: u32,
}

pub trait GI: Send + Sync + Copy {
    fn new_probe_index(&self, offset: IVec3, level: u8, pop_mask: u64, children: &[VoxelDAG64Node]) -> u32; 
}

impl GIPool {
    pub fn new(levels: usize) -> Self {
        
        let mut pools = vec![];

        for level in (0..levels).rev() {
            pools.push(sharded_slab::Slab::new());
        }

        Self {
            pools,
            search_order: search_order(),
        }
    }

    
    pub fn get_memory_size(&self) -> usize {
        32
    }
}

impl<'a> GI for GIExecutor<'a> {
    fn new_probe_index(&self, offset: IVec3, level: u8, pop_mask: u64, children: &[VoxelDAG64Node]) -> u32 {
        if pop_mask == 0 || level < GI_PROBE_MIN_LEVEL {
            return GI_PROBE_INDEX_NONE;
        }

        let pos = self.find_position(offset, level, pop_mask, children);
        if pos.is_none() {
            return GI_PROBE_INDEX_NONE;
        }

        let gi_level = &self.pool.pools[level as usize];
        gi_level.insert(GIProbe {
            position: pos.unwrap(),
            object_index: self.object_index
        }).expect(&format!("Probe pool full at level: {level}")) as u32
    }
}

impl<'a> GIExecutor<'a> {
    pub fn new(pool: &'a GIPool, object_index: u32) -> Self {
        Self {
            pool,
            object_index,
        }
    }

    fn find_position(&self, offset: IVec3, level: u8, pop_mask: u64, children: &[VoxelDAG64Node]) -> Option<IVec3> {

        let child_size = get_voxel_size(level - 1);
        if pop_mask != u64::MAX {
            for (i, pos) in self.pool.search_order {
                if pop_mask >> i & 1 == 0 {
                    return Some(offset + pos * child_size + child_size / 2);
                } 
            }
            unreachable!();
        }

        // No empty children we search one level down
        let child_size_2 = get_voxel_size(level - 2);
        for (i, pos) in search_order() {
            let child = children[i];

            if child.pop_mask == u64::MAX {
                continue;
            }

            for (i, pos_2) in self.pool.search_order {
                if child.pop_mask >> i & 1 == 0 {
                    return Some(offset + pos * child_size + pos_2 * child_size_2 + child_size_2 / 2);
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
    for (i, pos) in get_dag_node_children_i().into_iter().enumerate() {
        let dist = pos.as_vec3().distance(center);

        positions.push((dist, pos, i));
    }

    positions.sort_by(|a, b| a.0.total_cmp(&b.0)); 
    let positions: Vec<_> = positions.into_iter().map(|a| (a.2, a.1)).collect();
    positions.try_into().unwrap()
}

