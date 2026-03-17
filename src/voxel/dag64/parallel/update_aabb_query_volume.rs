use octa_force::{anyhow, glam::{vec3a, IVec3, UVec3, Vec3A, Vec4Swizzles}, log::debug, OctaResult};
use rayon::prelude::*;
use smallvec::{SmallVec, ToSmallVec};

use crate::{new_logic_state, util::{aabb::AABB, math::get_dag_node_children_i, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeChangeBounds, VolumeQureyAABB}, voxel::dag64::{entry::DAG64EntryKey, lod_heuristic::LODHeuristicT, node::VoxelDAG64Node, parallel::{MIN_PAR_LEVEL, ParallelVoxelDAG64}, util::get_voxel_size}};

impl ParallelVoxelDAG64 {

    pub fn update_aabb_query_volume<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3> + VolumeChangeBounds<V, T, 3> + Send + Sync, LOD: LODHeuristicT>(
        &mut self, 
        model: &M,
        lod: &LOD,
        based_on_entry: DAG64EntryKey,
    ) -> DAG64EntryKey {
        let change_aabb = model.get_change_bounds();
        let mut entry_data = self.expand_to_include_aabb(based_on_entry, change_aabb);

        let root = self.update_aabb_recursive_par(model, lod, change_aabb, entry_data.levels, entry_data.offset, entry_data.root_index);
        entry_data.root_index = self.nodes.push(&[root]);

        let key = self.entry_points.lock().insert(entry_data);

        key
    }
    
    pub(super) fn update_aabb_recursive_par<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3> + Send + Sync, LOD: LODHeuristicT>(
        &self, 
        model: &M, 
        lod: &LOD,
        aabb: AABB<V, T, 3>, 
        level: u8, 
        offset: IVec3, 
        index: u32
    ) -> VoxelDAG64Node {
        let node = self.nodes.get(index);

        if node.is_leaf() {
            let new_node = self.add_aabb_query_leaf(
                model, 
                offset,
                level,
            );

            return new_node;
        }
        
        let new_level = level -1;
        let new_size = get_voxel_size(new_level);

        let new_children: SmallVec<[_; 64]> = get_dag_node_children_i()
            .into_par_iter()
            .enumerate()
            .map(|(i, pos)| {
                let min = offset + pos * new_size;
                let max = min + new_size;
                let node_aabb = AABB::new(V::ve_from(min), V::ve_from(max));

                if !aabb.collides_aabb(node_aabb) {
                    return (None, false);
                }

                let index_in_children = node.get_index_in_children_unchecked(i as u32);
                let new_node = if !node.is_occupied(i as u32) {
                   
                    if new_level > MIN_PAR_LEVEL {
                        self.add_aabb_query_recursive_par(
                            model, 
                            lod,
                            node_aabb.min().ve_into(),
                            new_level,
                        )
                    } else {
                        self.add_aabb_query_recursive(
                            model, 
                            lod,
                            node_aabb.min().ve_into(),
                            new_level,
                        )
                    }
                    
                } else if aabb.contains_aabb(node_aabb) {

                    if new_level > MIN_PAR_LEVEL {
                        self.add_aabb_query_recursive_par(
                            model,
                            lod,
                            node_aabb.min().ve_into(),
                            new_level,
                        )
                    } else {
                        self.add_aabb_query_recursive(
                            model,
                            lod,
                            node_aabb.min().ve_into(),
                            new_level,
                        )
                    }

                } else {

                    if new_level > MIN_PAR_LEVEL {
                        self.update_aabb_recursive_par(
                            model,
                            lod,
                            aabb,
                            new_level,
                            node_aabb.min().ve_into(),
                            node.index() + index_in_children,
                        )
                    } else {
                        self.update_aabb_recursive(
                            model,
                            lod,
                            aabb,
                            new_level,
                            node_aabb.min().ve_into(),
                            node.index() + index_in_children,
                        )
                    }
                };

                (new_node.check_empty(), true)
            })
            .fold(|| SmallVec::new(), |mut vec, n|{
                vec.push(n);
                vec
            })
            .reduce(|| SmallVec::new(), |mut vec_a, vec_b| {
                vec_a.extend_from_slice(&vec_b);
                vec_a
            });

        let mut bitmask = node.pop_mask;
        let mut children: SmallVec<[_; 64]> = self.nodes.get_range(node.range()).to_smallvec();
        let mut j = children.len();

        for (i, (new_child, checked)) in new_children.into_iter().enumerate().rev() {
            let child_present = node.is_occupied(i as u32);

            if let Some(new_child) = new_child {

                if child_present {
                    j -= 1;
                    children[j] = new_child;
                } else {
                    children.insert(j, new_child);
                    bitmask |= 1 << i as u64;
                }
            } else if child_present {

                j -= 1;
                if checked {
                    children.remove(j);
                    bitmask &= !(1 << i as u64);
                }
            }
        }

        VoxelDAG64Node::single(
            false, 
            self.nodes.push(&children) as u32, 
            bitmask)
    }

    fn update_aabb_recursive<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3>, LOD: LODHeuristicT>(
        &self, 
        model: &M, 
        lod: &LOD,
        aabb: AABB<V, T, 3>, 
        level: u8, 
        offset: IVec3, 
        index: u32
    ) -> VoxelDAG64Node {
        let node = self.nodes.get(index);

        if node.is_leaf() {
            let new_node = self.add_aabb_query_leaf(
                model, 
                offset,
                level,
            );

            return new_node;
        }

        let mut new_children: SmallVec<[_; 64]> = self.nodes.get_range(node.range()).to_smallvec();
        let mut new_bitmask = node.pop_mask;
        
        let new_level = level -1;
        let new_size = get_voxel_size(new_level);

        let j = new_children.len(); 
        for (i, pos) in get_dag_node_children_i().into_iter().enumerate().rev() {
            let min = offset + pos * new_size;
            let max = min + new_size;
            let node_aabb = AABB::new(V::ve_from(min), V::ve_from(max));

            if !aabb.collides_aabb(node_aabb) {
                continue;
            }

            let index_in_children = node.get_index_in_children_unchecked(i as u32);
            if node.is_occupied(i as u32) {

                let new_child_node = if aabb.contains_aabb(node_aabb) {
                    self.add_aabb_query_recursive(
                        model, 
                        lod, 
                        min,
                        new_level,
                    )
                } else {
                    self.update_aabb_recursive(
                        model,
                        lod,
                        aabb,
                        new_level,
                        min,
                        node.index() + index_in_children,
                    )
                };

                if new_child_node.is_empty() {
                    new_children.remove(index_in_children as usize);
                    new_bitmask &= !(1 << i as u64);
                    
                } else {
                    new_children[index_in_children as usize] = new_child_node;
                }

                continue;
            }

            let new_child_node = self.add_aabb_query_recursive(
                model,
                lod,
                min,
                new_level,
            );

            if new_child_node.is_empty() {
                continue;
            }

            new_children.insert(index_in_children as usize, new_child_node);
            new_bitmask |= 1 << i as u64; 
        }

        VoxelDAG64Node::single(
            false, 
            self.nodes.push(&new_children) as u32, 
            new_bitmask)
    }
}
