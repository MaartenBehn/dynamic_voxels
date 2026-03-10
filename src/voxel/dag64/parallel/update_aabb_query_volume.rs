use octa_force::{anyhow, glam::{vec3a, IVec3, UVec3, Vec3A, Vec4Swizzles}, log::debug, OctaResult};
use rayon::prelude::*;
use smallvec::{SmallVec, ToSmallVec};

use crate::{new_logic_state, util::{aabb::AABB, math::get_dag_node_children_xzy_i, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeChangeBounds, VolumeQureyAABB}, voxel::dag64::{lod_heuristic::LODHeuristicT, node::VoxelDAG64Node}};

use super::{DAG64Entry, DAG64EntryKey, ParallelVoxelDAG64, VoxelDAG64};


impl ParallelVoxelDAG64 {

    pub fn update_aabb_query_volume<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3> + VolumeChangeBounds<V, T, 3> + Send + Sync, LOD: LODHeuristicT>(
        &mut self, 
        model: &M,
        lod: &LOD,
        based_on_entry: DAG64EntryKey,
    ) -> OctaResult<DAG64EntryKey> {
        let change_aabb = model.get_change_bounds();
        let mut entry_data = self.expand_to_include_aabb(based_on_entry, change_aabb)?;

        let root = self.update_aabb_recursive_par(model, lod, change_aabb, entry_data.levels, entry_data.offset, entry_data.root_index)?;
        entry_data.root_index = self.nodes.push(&[root])?;

        let key = self.entry_points.lock().insert(entry_data);

        Ok(key)
    }
    
    pub(super) fn update_aabb_recursive_par<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3> + Send + Sync, LOD: LODHeuristicT>(
        &self, 
        model: &M, 
        lod: &LOD,
        aabb: AABB<V, T, 3>, 
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
        let new_scale = 1 << (2 * new_level);

        let new_children: Vec<_> = get_dag_node_children_xzy_i()
            .into_par_iter()
            .enumerate()
            .map(|(i, pos)| {
                let min = offset + pos * new_scale;
                let max = min + new_scale;
                let node_aabb = AABB::new(V::from_ivec3(min), V::from_ivec3(max));

                if !aabb.collides_aabb(node_aabb) {
                    return Ok(None);
                }

                let index_in_children = node.get_index_in_children_unchecked(i as u32);
                let new_node = if !node.is_occupied(i as u32) {

                    self.add_aabb_query_recursive_par(
                        model, 
                        lod,
                        node_aabb.min().to_ivec3(),
                        new_level,
                    )?.check_empty()

                } else if aabb.contains_aabb(node_aabb) {
                    Some(self.add_aabb_query_recursive_par(
                        model,
                        lod,
                        node_aabb.min().to_ivec3(),
                        new_level,
                    )?)
                } else {
                    Some(self.update_aabb_recursive_par(
                        model,
                        lod,
                        aabb,
                        new_level,
                        node_aabb.min().to_ivec3(),
                        node.ptr() + index_in_children,
                    )?)
                };

                Ok(new_node)
            })
            .collect();

        if !new_children.is_empty() {
            let mut bitmask = node.pop_mask;
            let mut children: SmallVec<[_; 64]> = self.nodes.get_range(node.range()).to_smallvec();
            let mut j = children.len();

            for (i, new_child) in new_children.into_iter().enumerate().rev() {
                if new_child.is_err() {
                    return Err(new_child.unwrap_err());
                }

                let child_present = (bitmask & (1 << i)) != 0;

                if let Some(new_child) = new_child.unwrap() {

                    if child_present {
                        j -= 1;
                        children[j] = new_child;
                    } else {
                        children.insert(j, new_child);
                        bitmask |= 1 << i as u64;
                    }
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

    fn update_aabb_recursive<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3>, LOD: LODHeuristicT>(
        &self, 
        model: &M, 
        lod: &LOD,
        aabb: AABB<V, T, 3>, 
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
            let node_aabb = AABB::new(V::from_ivec3(min), V::from_ivec3(max));

            if aabb.collides_aabb(node_aabb) {

                let index_in_children = node.get_index_in_children_unchecked(i as u32);
                if !node.is_occupied(i as u32) {

                    let new_child_node = self.add_aabb_query_recursive(
                        model,
                        lod,
                        min,
                        new_level,
                    )?;

                    if new_child_node.is_empty() {
                        continue;
                    }

                    if new_children.is_empty() {
                        new_children = self.nodes.get_range(node.range()).to_smallvec();
                    }

                    new_children.insert(index_in_children as usize, new_child_node);
                    new_bitmask |= 1 << i as u64; 

                    continue;
                } 

                let new_child_node = if aabb.contains_aabb(node_aabb) {
                    self.add_aabb_query_recursive(
                        model, 
                        lod, 
                        min,
                        new_level,
                    )?
                } else {
                    self.update_aabb_recursive(
                        model,
                        lod,
                        aabb,
                        new_level,
                        min,
                        node.ptr() + index_in_children,
                    )?
                };

                if new_children.is_empty() {
                    new_children = self.nodes.get_range(node.range()).to_smallvec();
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
