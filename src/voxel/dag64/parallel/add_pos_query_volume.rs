use itertools::Either;
use octa_force::{anyhow::{self, anyhow}, glam::{IVec3, Vec3Swizzles}, OctaResult};
use smallvec::SmallVec;
use crate::{gi::gi_pool::{GI, GI_PROBE_MIN_LEVEL, GIPool}, util::{math::{get_dag_node_children, get_dag_node_children_i}, math_config::MC, number::Nu, vector::Ve}, volume::VolumeQureyPosValue, voxel::dag64::{entry::{DAG64Entry, DAG64EntryKey}, lod_heuristic::LODHeuristicT, node::VoxelDAG64Node, util::{get_dag_offset_levels, get_voxel_size}}};
use super::ParallelVoxelDAG64;
use rayon::iter::{walk_tree_postfix};
use rayon::prelude::*;


impl ParallelVoxelDAG64 {
    pub fn add_pos_query_volume<G: GI, V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3> + Sync + Send, LOD: LODHeuristicT>(
        &mut self, 
        model: &M,
        lod: &LOD,
        gi: G,
    ) -> DAG64EntryKey {
        let (offset, levels) = get_dag_offset_levels(model);
        if levels == 0 {
            return self.empty_entry();
        }

        let root = self.add_pos_query_recursive_par(model, lod, gi, offset, levels);

        let root_index = self.nodes.push(&[root]);
        let key = self.entry_points.lock().insert(DAG64Entry { 
            levels, 
            root_index, 
            offset, 
        });

        key
    }

    pub(super) fn add_pos_query_recursive_par<G: GI, V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3> + Sync + Send, LOD: LODHeuristicT>(
        &self,
        model: &M,
        lod: &LOD,
        gi: G,
        offset: IVec3,
        level: u8,
    ) -> VoxelDAG64Node {
        if level <= lod.lod_level(offset) {
            self.add_pos_query_leaf(model, offset, level)
        } else { 
            let new_level = level - 1;
            let new_size = get_voxel_size(new_level);
            let (children, pop_mask) = get_dag_node_children_i().into_par_iter()
                .enumerate()
                .map(move |(i, pos)| {
                    let pos = offset + pos * new_size;
                    let res = self.add_pos_query_recursive(
                        model, 
                        lod,
                        gi,
                        pos, 
                        new_level);

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

            let index = self.nodes.push(&children);
            let gi_index = gi.new_probe_index(offset, level, pop_mask, &children);
            VoxelDAG64Node::new(false, index, pop_mask, gi_index)
        }
    }

    pub(super) fn add_pos_query_recursive<G: GI, V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3>, LOD: LODHeuristicT>(
        &self,
        model: &M,
        lod: &LOD,
        gi_pool: G,
        offset: IVec3,
        level: u8,
    ) -> VoxelDAG64Node {
        if level <= lod.lod_level(offset) {
            self.add_pos_query_leaf(model, offset, level)
        } else {
            let new_level = level -1;
            let new_scale = get_voxel_size(new_level);
            let mut children = SmallVec::<[_; 64]>::new();
            let mut pop_mask = 0;

            for (i, pos) in get_dag_node_children_i().into_iter().enumerate() {
                let child = self.add_pos_query_recursive(
                    model,
                    lod,
                    gi_pool,
                    offset + pos * new_scale,
                    new_level,
                );
                if !child.is_empty() {
                    children.push(child);
                    pop_mask |= 1 << i  as u64;
                }
            }

            let index = self.nodes.push(&children);
            let gi_index = gi_pool.new_probe_index(offset, level, pop_mask, &children);
            VoxelDAG64Node::new(false, index, pop_mask, gi_index)
        }
    }

    pub(super) fn add_pos_query_leaf<V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3>>(
        &self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> VoxelDAG64Node {
        let mut vec = SmallVec::<[_; 64]>::new();
        let mut bitmask = 0;

        for (i, pos) in get_dag_node_children_i().into_iter().enumerate() {
            let pos = offset + pos;
            let value = model.get_value(V::ve_from(pos));

            if value != 0 {
                vec.push(value);
                bitmask |= 1 << i as u64;
            }
        } 

        let ptr = self.data.push(&vec);
        VoxelDAG64Node::single(true, ptr, bitmask)
    }
}

