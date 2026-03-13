use itertools::Either;
use octa_force::{anyhow::{self, anyhow}, glam::{IVec3, Vec3Swizzles}, OctaResult};
use smallvec::SmallVec;
use crate::{util::{aabb::AABB, math::{get_dag_node_children, get_dag_node_children_xzy_i}, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeQureyAABB, VolumeQureyAABBResult}, voxel::dag64::{DAG64Entry, DAG64EntryKey, lod_heuristic::LODHeuristicT, node::VoxelDAG64Node, parallel::MIN_PAR_LEVEL, util::get_dag_offset_levels}};
use super::ParallelVoxelDAG64;
use rayon::iter::{walk_tree_postfix};
use rayon::prelude::*;


impl ParallelVoxelDAG64 {
    pub fn add_aabb_query_volume<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T,3> + Send + Sync, LOD: LODHeuristicT>(
        &mut self, 
        model: &M,
        lod: &LOD,
    ) -> DAG64EntryKey {
        let (offset, levels) = get_dag_offset_levels(model);
        if levels == 0 {
            return self.empty_entry();
        }

        let root = self.add_aabb_query_recursive_par(model, lod, offset, levels);

        let root_index = self.nodes.push(&[root]);
        let key = self.entry_points.lock().insert(DAG64Entry { 
            levels, 
            root_index, 
            offset, 
        });

        key
    }
    
    pub(super) fn add_aabb_query_recursive_par<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3> + Send + Sync, LOD: LODHeuristicT>(
        &self,
        model: &M,
        lod: &LOD,
        offset: IVec3,
        node_level: u8,
    ) -> VoxelDAG64Node {
        if node_level <= lod.lod_level(offset) {
            self.add_aabb_query_leaf(model, offset, node_level)
        } else {

            let scale = 1 << (2 * node_level);
            let aabb = AABB::new(
                V::ve_from(offset), 
                V::ve_from(offset + scale));

            let res = model.get_aabb_value(aabb); 

            match res {
                VolumeQureyAABBResult::Full(v) => {
                    if v == 0 {
                        VoxelDAG64Node::new(true, 0, 0)
                    } else {
                        VoxelDAG64Node::new(true, self.data.push(&[v; 64]) as u32, u64::MAX)
                    }
                },
                VolumeQureyAABBResult::Mixed =>  {
                    let new_level = node_level - 1;
                    let new_scale = 1 << (2 * new_level);

                    let (vec, bitmask) = get_dag_node_children_xzy_i().into_par_iter()
                        .enumerate()
                        .map(move |(i, pos)| {
                            let pos = offset + pos * new_scale;
                            let res = if node_level > MIN_PAR_LEVEL {
                                self.add_aabb_query_recursive_par(
                                    model,
                                    lod,
                                    pos, 
                                    new_level) 
                            } else {
                                self.add_aabb_query_recursive(
                                    model,
                                    lod,
                                    pos, 
                                    new_level) 
                            };

                            if res.is_empty() {
                                None
                            } else {
                                Some((i, res))
                            }
                        })
                        .flatten()
                        .fold(|| (SmallVec::<[_; 64]>::new(), 0_u64), 
                            |(mut vec, mut bitmask), a| {
                                let (i, n) = a;
                                vec.push(n);
                                bitmask |= 1 << i;
                                (vec, bitmask)
                            })
                        .reduce(|| (SmallVec::<[_; 64]>::new(), 0_u64), 
                            |(mut vec_a, mut bitmask_a), (vec_b, bitmask_b)| {
                                vec_a.extend_from_slice(&vec_b);
                                bitmask_a |= bitmask_b;
                                (vec_a, bitmask_a)
                            });

                    let ptr = self.nodes.push(&vec);
                    VoxelDAG64Node::new(false, ptr, bitmask)
                },
            }
        }
    }

    pub(super) fn add_aabb_query_recursive<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3>, LOD: LODHeuristicT>(
        &self,
        model: &M,
        lod: &LOD,
        offset: IVec3,
        node_level: u8,
    ) -> VoxelDAG64Node {
        if node_level <= lod.lod_level(offset) {
            self.add_aabb_query_leaf(model, offset, node_level)
        } else {
            let scale = 1 << (2 * node_level);
            let aabb = AABB::new(
                V::ve_from(offset), 
                V::ve_from(offset + scale));

            let res = model.get_aabb_value(aabb); 

            match res {
                VolumeQureyAABBResult::Full(v) => {
                    if v == 0 {
                        VoxelDAG64Node::new(true, 0, 0)
                    } else {
                        VoxelDAG64Node::new(true, self.data.push(&[v; 64]) as u32, u64::MAX)
                    }
                },
                VolumeQureyAABBResult::Mixed =>  {
                    let new_level = node_level -1;
                    let new_scale = 1 << (2 * new_level);
                    
                    let mut nodes = SmallVec::<[_; 64]>::new();
                    let mut bitmask = 0;

                    for (i, pos) in get_dag_node_children_xzy_i().into_iter().enumerate() {
                        let child = self.add_aabb_query_recursive(
                            model,
                            lod,
                            offset + pos * new_scale,
                            new_level,
                        );
                        if !child.is_empty() {
                            nodes.push(child);
                            bitmask |= 1 << i as u64;
                        }
                    }

                    VoxelDAG64Node::new(false, self.nodes.push(&nodes) as u32, bitmask)
                },
            }
        }
    }

    pub(super) fn add_aabb_query_leaf<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3>>(
        &self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> VoxelDAG64Node {
        let scale = 1 << (2 * node_level);
        let aabb = AABB::new(
                V::ve_from(offset), 
                V::ve_from(offset + scale));

        let res = model.get_aabb_value(aabb);

        match res {
            VolumeQureyAABBResult::Full(v) => {
                if v == 0 {
                    VoxelDAG64Node::new(true, 0, 0)
                } else {
                    VoxelDAG64Node::new(true, self.data.push(&[v; 64]) as u32, u64::MAX)
                }
            },
            VolumeQureyAABBResult::Mixed =>  {
                self.add_pos_query_leaf(model, offset, node_level)
            },
        }
    }
}

