use octa_force::{anyhow, glam::{vec3a, IVec3, UVec3, Vec3A, Vec4Swizzles}, log::debug, OctaResult};
use rayon::prelude::*;
use smallvec::{SmallVec, ToSmallVec};

use crate::{multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, new_logic_state, util::{aabb3d::AABB, iaabb3d::AABBI, math::get_dag_node_children_xzy_i}, volume::{VolumeQureyAABBI, VolumeQureyPosValueI}, voxel::dag64::node::VoxelDAG64Node};

use super::{DAG64Entry, DAG64EntryKey, ParallelVoxelDAG64, VoxelDAG64};


impl ParallelVoxelDAG64 {
    pub fn update_aabb_query_volume<M: VolumeQureyAABBI + Send + Sync>(
        &mut self, 
        model: &M,
        based_on_entry: DAG64EntryKey,
    ) -> OctaResult<DAG64EntryKey> {
        let model_aabb = model.get_bounds_i().into();
        let mut entry_data = self.expand_to_include_aabb(based_on_entry, model_aabb)?;

        let root = self.update_aabb_recursive_par(model, model_aabb, entry_data.levels, entry_data.offset, entry_data.root_index)?;
        entry_data.root_index = self.nodes.push(&[root])?;

        let key = self.entry_points.lock().insert(entry_data);

        Ok(key)
    }

    fn update_aabb_recursive_par<M: VolumeQureyAABBI + Send + Sync>(
        &self, 
        model: &M, 
        aabb: AABBI, 
        node_level: u8, 
        offset: IVec3, 
        index: u32
    ) -> OctaResult<VoxelDAG64Node> {
        let node = self.nodes.get(index);

        if node.is_leaf() {
            let new_node = self.add_aabb_query_leaf(
                model, 
                offset,
                node_level,
            )?;

            return Ok(new_node);
        }
        
        let new_level = node_level -1;
        let new_scale = 4_i32.pow(new_level as u32);

        let (new_children, new_bitmask) = get_dag_node_children_xzy_i()
            .into_par_iter()
            .enumerate()
            .map(|(i, pos)| {
                let min = offset + pos * new_scale;
                let max = min + new_scale;
                (i, AABBI::new(min, max))
            })
            .filter(|(_, node_aabb)| aabb.collides_aabb(*node_aabb))
            .map(|(i, node_aabb)| {
                let index_in_children = node.get_index_in_children_unchecked(i as u32);
                let new_node = if !node.is_occupied(i as u32) {

                    (self.add_aabb_query_recursive(
                        model, 
                        node_aabb.min,
                        new_level,
                    )?.check_empty(), true)
                } else if aabb.contains_aabb(node_aabb) {
                    (Some(self.add_aabb_query_recursive(
                        model, 
                        node_aabb.min,
                        new_level,
                    )?), false)
                } else {
                    (Some(self.update_aabb_recursive(
                        model,
                        aabb,
                        new_level,
                        node_aabb.min,
                        node.ptr() + index_in_children,
                    )?), false)
                };

                Ok::<_, anyhow::Error>((i, new_node))
            })
            .try_fold(|| (SmallVec::<[_; 64]>::new(), 0_u64), 
                |(mut children, mut bitmask), a| {
                    let (i, (new_node, insert)) = a?;
                    if let Some(new_node) = new_node  {
                        children.push(new_node);
                        bitmask |= 1 << i;
                    }
                    Ok::<_, anyhow::Error>((children, bitmask))
                })
            .try_reduce(|| (SmallVec::new(), 0_u64), 
                |(mut children_a, mut bitmask_a), (children_b, bitmask_b)| {
                    children_a.extend_from_slice(&children_b);
                    bitmask_a |= bitmask_b;
                    Ok((children_a, bitmask_a))
                })?;



        if !new_children.is_empty() {
            let mut bitmask = node.pop_mask;
            let mut children = self.nodes.get_range::<64>(node.range());
            let mut j = children.len();
            let mut k = new_children.len();

            for i in (0..64).rev() {
                let child_present = (bitmask & (1 << i)) != 0;
                let new_child_present = (new_bitmask & (1 << i)) != 0; 

                if child_present && new_child_present {
                    j -= 1;
                    k -= 1;
                    children[j] = new_children[k];
                } else if new_child_present {
                    k -= 1;
                    children.insert(j, new_children[k]);
                    bitmask |= 1 << i as u64;
                } else if child_present {
                    j -= 1;
                }
            }

            let new_node = VoxelDAG64Node::new(
                false, 
                self.nodes.push(&children)? as u32, 
                bitmask);

            Ok(new_node)
        } else {
            Ok(node)
        }
    }

    fn update_aabb_recursive<M: VolumeQureyAABBI>(
        &self, 
        model: &M, 
        aabb: AABBI, 
        node_level: u8, 
        offset: IVec3, 
        index: u32
    ) -> OctaResult<VoxelDAG64Node> {
        let node = self.nodes.get(index);

        if node.is_leaf() {
            let new_node = self.add_aabb_query_leaf(
                model, 
                offset,
                node_level,
            )?;

            return Ok(new_node);
        }

        let mut new_children: SmallVec<[_; 64]> = SmallVec::new();
        let mut new_bitmask = node.pop_mask;
        
        let new_level = node_level -1;
        let new_scale = 4_i32.pow(new_level as u32);
        for (i, pos) in get_dag_node_children_xzy_i().into_iter()
            .enumerate()
            .rev() {
            let min = offset + pos * new_scale;
            let max = min + new_scale;
            let node_aabb = AABBI::new(min, max);

            if aabb.collides_aabb(node_aabb) {

                let index_in_children = node.get_index_in_children_unchecked(i as u32);
                if !node.is_occupied(i as u32) {

                    let new_child_node = self.add_aabb_query_recursive(
                        model, 
                        min,
                        new_level,
                    )?;

                    if new_child_node.is_empty() {
                        continue;
                    }

                    if new_children.is_empty() {
                        new_children = self.nodes.get_range::<64>(node.range());
                    }

                    new_children.insert(index_in_children as usize, new_child_node);
                    new_bitmask |= 1 << i as u64; 

                    continue;
                } 

                let new_child_node = if aabb.contains_aabb(node_aabb) {
                    self.add_aabb_query_recursive(
                        model, 
                        min,
                        new_level,
                    )?
                } else {
                    self.update_aabb_recursive(
                        model,
                        aabb,
                        new_level,
                        min,
                        node.ptr() + index_in_children,
                    )?
                };

                if new_children.is_empty() {
                    new_children = self.nodes.get_range::<64>(node.range());
                }
                new_children[index_in_children as usize] = new_child_node;
            }           
        }

        if !new_children.is_empty() {
            let new_node = VoxelDAG64Node::new(
                false, 
                self.nodes.push(&new_children)? as u32, 
                new_bitmask);

            Ok(new_node)
        } else {
            Ok(node)
        }
    }
}
