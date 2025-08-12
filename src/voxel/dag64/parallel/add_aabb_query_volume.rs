use itertools::Either;
use octa_force::{anyhow::{self, anyhow}, glam::{IVec3, Vec3Swizzles}, OctaResult};
use smallvec::SmallVec;
use crate::{util::{iaabb3d::AABBI, math::{get_dag_node_children, get_dag_node_children_xzy_i}}, volume::{VolumeQureyAABBI, VolumeQureyAABBResult}, voxel::dag64::{node::VoxelDAG64Node, util::get_dag_offset_levels, DAG64Entry, DAG64EntryKey}};
use super::ParallelVoxelDAG64;
use rayon::iter::{walk_tree_postfix};
use rayon::prelude::*;


impl ParallelVoxelDAG64 {
    pub fn add_aabb_query_volume<M: VolumeQureyAABBI + Sync + Send>(&mut self, model: &M) -> OctaResult<DAG64EntryKey> {
        let (offset, levels) = get_dag_offset_levels(model);

        let root = self.add_aabb_query_recursive_par(model, offset, levels)?;

        let root_index = self.nodes.push(&[root])?;
        let key = self.entry_points.lock().insert(DAG64Entry { 
            levels, 
            root_index, 
            offset, 
        });

        Ok(key)
    }

    pub fn add_aabb_query_recursive_par<M: VolumeQureyAABBI + Sync + Send>(
        &self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> OctaResult<VoxelDAG64Node> {
        if node_level == 1 {
            self.add_aabb_query_leaf(model, offset, node_level)
        } else {
            let scale = 4_i32.pow(node_level as u32);
            let aabb = AABBI::new(
                offset, 
                offset + scale);

            let res = model.get_aabb_value_i(aabb); 

            match res {
                VolumeQureyAABBResult::Full(v) => {
                    if v == 0 {
                        Ok(VoxelDAG64Node::new(true, 0, 0))
                    } else {
                        Ok(VoxelDAG64Node::new(true, self.data.push(&[v; 64])? as u32, u64::MAX))
                    }
                },
                VolumeQureyAABBResult::Mixed =>  {
                    let new_level = node_level - 1;
                    let new_scale = 4_i32.pow(new_level  as u32);
                    let (vec, bitmask) = get_dag_node_children_xzy_i().into_par_iter()
                        .enumerate()
                        .map(move |(i, pos)| {
                            let pos = offset + pos * new_scale;
                            let res = self.add_aabb_query_recursive(
                                model, 
                                pos, 
                                new_level);

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

                    let ptr = self.nodes.push(&vec)?;
                    Ok(VoxelDAG64Node::new(false, ptr, bitmask))
                },
            }
        }
    }

    pub fn add_aabb_query_recursive<M: VolumeQureyAABBI>(
        &self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> OctaResult<VoxelDAG64Node> {
        if node_level == 1 {
            self.add_aabb_query_leaf(model, offset, node_level)
        } else {
            let scale = 4_i32.pow(node_level as u32);
            let aabb = AABBI::new(
                offset, 
                offset + scale);

            let res = model.get_aabb_value_i(aabb); 

            match res {
                VolumeQureyAABBResult::Full(v) => {
                    if v == 0 {
                        Ok(VoxelDAG64Node::new(true, 0, 0))
                    } else {
                        Ok(VoxelDAG64Node::new(true, self.data.push(&[v; 64])? as u32, u64::MAX))
                    }
                },
                VolumeQureyAABBResult::Mixed =>  {
                    let new_level = node_level -1;
                    let new_scale = 4_i32.pow(new_level as u32);
                    let mut nodes = SmallVec::<[_; 64]>::new();
                    let mut bitmask = 0;

                    for (i, pos) in get_dag_node_children_xzy_i().into_iter().enumerate() {
                        let child = self.add_aabb_query_recursive(
                            model,
                            offset + pos * new_scale,
                            new_level,
                        )?;
                        if !child.is_empty() {
                            nodes.push(child);
                            bitmask |= 1 << i as u64;
                        }
                    }

                    Ok(VoxelDAG64Node::new(false, self.nodes.push(&nodes)? as u32, bitmask))
                },
            }
        }
    }

    pub fn add_aabb_query_leaf<M: VolumeQureyAABBI>(
        &self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> OctaResult<VoxelDAG64Node> {
        let scale = 4_i32.pow(node_level as u32);
        let aabb = AABBI::new(
            offset, 
            offset + scale);

        let res = model.get_aabb_value_i(aabb);

        match res {
            VolumeQureyAABBResult::Full(v) => {
                if v == 0 {
                    Ok(VoxelDAG64Node::new(true, 0, 0))
                } else {
                    Ok(VoxelDAG64Node::new(true, self.data.push(&[v; 64])? as u32, u64::MAX))
                }
            },
            VolumeQureyAABBResult::Mixed =>  {
                self.add_pos_query_leaf(model, offset, node_level)
            },
        }
    }
}

