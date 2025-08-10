use itertools::Either;
use octa_force::{anyhow::{self, anyhow}, glam::{IVec3, Vec3Swizzles}, OctaResult};
use smallvec::SmallVec;
use crate::{util::math::get_dag_node_children, volume::VolumeQureyPosValueI, voxel::dag64::{node::VoxelDAG64Node, util::get_dag_offset_levels, DAG64EntryData, DAG64EntryKey}};
use super::ParallelVoxelDAG64;
use rayon::iter::{walk_tree_postfix};
use rayon::prelude::*;

const PARALLSE_LEAFS: bool = false;

impl ParallelVoxelDAG64 {
    pub fn add_pos_query_volume<M: VolumeQureyPosValueI + Sync + Send>(&mut self, model: &M) -> OctaResult<DAG64EntryKey> {
        let (offset, levels) = get_dag_offset_levels(model);

        let root = self.add_pos_query_recursive_par(model, offset, levels)?;

        let root_index = self.nodes.push_single(root).result_blocking()?;
        let key = self.entry_points.lock().insert(DAG64EntryData { 
            levels, 
            root_index, 
            offset, 
        });


        Ok(key)
    }

    pub fn add_pos_query_recursive_par<M: VolumeQureyPosValueI + Sync + Send>(
        &self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> OctaResult<VoxelDAG64Node> {
        if node_level == 1 {

            let (vec, bitmask) = if PARALLSE_LEAFS {
                get_dag_node_children().into_par_iter()
                    .map(|pos| {
                        let pos = offset + pos.xyz().as_ivec3();
                        model.get_value_i(pos)
                    })
                    .enumerate()
                    .filter(|(_, v)| *v != 0)
                    .fold(|| (SmallVec::<[_; 64]>::new(), 0_u64), 
                        |(mut vec, mut bitmask), (i, v)| {
                            vec.push(v);
                            bitmask |= 1 << i;
                            (vec, bitmask)
                        })
                    .reduce(|| (SmallVec::<[_; 64]>::new(), 0_u64), 
                        |(mut vec_a, mut bitmask_a), (vec_b, bitmask_b)| {
                            vec_a.copy_from_slice(&vec_b);
                            bitmask_a |= bitmask_b;
                            (vec_a, bitmask_a)
                        })
            } else {
                let mut vec = SmallVec::<[_; 64]>::new();
                let mut bitmask = 0;

                for z in 0..4 {
                    for y in 0..4 {
                        for x in 0..4 {
                            // INFO: DAG Renderer works in XZY Space instead of XYZ like the rest of the
                            // engine
                            let pos = offset + IVec3::new(x, z, y);
                            let value = model.get_value_i(pos);

                            if value != 0 {
                                vec.push(value);
                                bitmask |= 1 << IVec3::new(x, y, z).dot(IVec3::new(1, 4, 16)) as u64;
                            }
                        }
                    }
                }

                (vec, bitmask)
            };
            
            let ptr = self.data.push(vec).result_blocking()?;
            Ok(VoxelDAG64Node::new(true, ptr, bitmask))
        } else {

            let new_level = node_level - 1;
            let new_scale = 4_i32.pow(new_level  as u32);
            let (vec, bitmask) = get_dag_node_children().into_par_iter()
                .enumerate()
                .map(|(i, pos)| {
                    let pos = offset + pos.xyz().as_ivec3() * new_scale;
                    let res = self.add_pos_query_recursive_par(model, pos, new_level);
                    if let Ok(res) = res {
                        if res.is_empty() {
                            None
                        } else {
                            Some(Ok((i, res)))
                        }
                    } else {
                        Some(Err(res.unwrap_err()))
                    }
                })
                .flatten()
                .try_fold(|| (SmallVec::<[_; 64]>::new(), 0_u64), 
                    |(mut vec, mut bitmask), a| {
                        let (i, n) = a?;
                        vec.push(n);
                        bitmask |= 1 << i;
                        Ok::<_, anyhow::Error>((vec, bitmask))
                    })
                .try_reduce(|| (SmallVec::<[_; 64]>::new(), 0_u64), 
                    |(mut vec_a, mut bitmask_a), (vec_b, bitmask_b)| {
                        vec_a.extend_from_slice(&vec_b);
                        bitmask_a |= bitmask_b;
                        Ok((vec_a, bitmask_a))
                    })?;

            let ptr = self.nodes.push(vec).result_blocking()?;
            Ok(VoxelDAG64Node::new(false, ptr, bitmask))
        }
    }
}

