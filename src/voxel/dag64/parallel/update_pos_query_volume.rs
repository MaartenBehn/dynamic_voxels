use octa_force::{anyhow, glam::{vec3a, IVec3, UVec3, Vec3A, Vec4Swizzles}, log::debug, OctaResult};
use rayon::prelude::*;
use smallvec::{SmallVec, ToSmallVec};

use crate::{gi::gi_pool::{GI, GIPool}, new_logic_state, util::{aabb::AABB, math::get_dag_node_children_xzy_i, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeChangeBounds, VolumeQureyPosValue}, voxel::dag64::{entry::DAG64EntryKey, lod_heuristic::LODHeuristicT, node::VoxelDAG64Node, parallel::ParallelVoxelDAG64, util::get_voxel_size}};


impl ParallelVoxelDAG64 {
    pub fn update_pos_query_volume<G: GI, V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3> + VolumeChangeBounds<V, T, 3> + Send + Sync, LOD: LODHeuristicT>(
        &mut self, 
        model: &M,
        lod: &LOD,
        gi: G,
        based_on_entry: DAG64EntryKey,
    ) -> DAG64EntryKey {
        let change_aabb = model.get_change_bounds();
        let mut entry_data = self.expand_to_include_aabb(based_on_entry, change_aabb);

        let root = self.update_pos_recursive_par(model, lod, gi, 
            change_aabb, entry_data.levels, entry_data.offset, entry_data.root_index);
        entry_data.root_index = self.nodes.push(&[root]);

        let key = self.entry_points.lock().insert(entry_data);

        key
    }

    pub(super) fn update_pos_recursive_par<G: GI, V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3> + Send + Sync, LOD: LODHeuristicT>(
        &self, 
        model: &M, 
        lod: &LOD,
        gi: G,
        aabb: AABB<V, T, 3>, 
        level: u8, 
        offset: IVec3, 
        index: u32
    ) -> VoxelDAG64Node {
        let node = self.nodes.get(index);

        if node.is_leaf() {
            let new_node = self.add_pos_query_leaf(
                model, 
                offset,
                level,
            );

            return new_node;
        }
        
        let new_level = level -1;
        let new_scale = get_voxel_size(level);

        let (new_children, new_bitmask) = get_dag_node_children_xzy_i()
            .into_par_iter()
            .enumerate()
            .map(|(i, pos)| {
                let min = offset + pos * new_scale;
                let max = min + new_scale;
                (i, AABB::new(V::ve_from(min), V::ve_from(max)))
            })
            .filter(|(_, node_aabb)| aabb.collides_aabb(*node_aabb))
            .map(|(i, node_aabb)| {
                let index_in_children = node.get_index_in_children_unchecked(i as u32);
                let new_node = if !node.is_occupied(i as u32) {

                    (self.add_pos_query_recursive(
                        model, 
                        lod,
                        gi,
                        node_aabb.min().ve_into(),
                        new_level,
                    ).check_empty(), true)
                } else if aabb.contains_aabb(node_aabb) {
                    (Some(self.add_pos_query_recursive(
                        model, 
                        lod,
                        gi,
                        node_aabb.min().ve_into(),
                        new_level,
                    )), false)
                } else {
                    (Some(self.update_pos_recursive(
                        model,
                        lod,
                        gi,
                        aabb,
                        new_level,
                        node_aabb.min().ve_into(),
                        node.index() + index_in_children,
                    )), false)
                };

                (i, new_node)
            })
            .fold(|| (SmallVec::<[_; 64]>::new(), 0_u64), 
                |(mut children, mut bitmask), a| {
                    let (i, (new_node, insert)) = a;
                    if let Some(new_node) = new_node  {
                        children.push(new_node);
                        bitmask |= 1 << i;
                    }
                    (children, bitmask)
                })
            .reduce(|| (SmallVec::new(), 0_u64), 
                |(mut children_a, mut bitmask_a), (children_b, bitmask_b)| {
                    children_a.extend_from_slice(&children_b);
                    bitmask_a |= bitmask_b;
                    (children_a, bitmask_a)
                });



        if !new_children.is_empty() {
            let mut pop_mask = node.pop_mask;
            let mut children: SmallVec<[_; 64]> = self.nodes.get_range(node.range()).to_smallvec();
            let mut j = children.len();
            let mut k = new_children.len();

            for i in (0..64).rev() {
                let child_present = (pop_mask & (1 << i)) != 0;
                let new_child_present = (new_bitmask & (1 << i)) != 0; 

                if child_present && new_child_present {
                    j -= 1;
                    k -= 1;
                    children[j] = new_children[k];
                } else if new_child_present {
                    k -= 1;
                    children.insert(j, new_children[k]);
                    pop_mask |= 1 << i as u64;
                } else if child_present {
                    j -= 1;
                }
            }

            let index = self.nodes.push(&children);
            let gi_index = gi.new_probe_index(offset, level, pop_mask, &children); 
            VoxelDAG64Node::new(false, index, pop_mask, gi_index)
        } else {
            node
        }
    }

    fn update_pos_recursive<G: GI, V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3>, LOD: LODHeuristicT>(
        &self, 
        model: &M, 
        lod: &LOD,
        gi: G,
        aabb: AABB<V, T, 3>, 
        level: u8, 
        offset: IVec3, 
        index: u32
    ) -> VoxelDAG64Node {
        let node = self.nodes.get(index);

        if node.is_leaf() {
            let new_node = self.add_pos_query_leaf(
                model, 
                offset,
                level,
            );

            return new_node;
        }

        let mut new_children: SmallVec<[_; 64]> = SmallVec::new();
        let mut new_pop_mask = node.pop_mask;
        
        let new_level = level -1;
        let new_size = get_voxel_size(new_level);
        
        for (i, pos) in get_dag_node_children_xzy_i().into_iter()
            .enumerate()
            .rev() {
            let min = offset + pos * new_size;
            let max = min + new_size;
            let node_aabb = AABB::new(V::ve_from(min), V::ve_from(max));

            if aabb.collides_aabb(node_aabb) {

                let index_in_children = node.get_index_in_children_unchecked(i as u32);
                if !node.is_occupied(i as u32) {

                    let new_child_node = self.add_pos_query_recursive(
                        model,
                        lod,
                        gi,
                        min,
                        new_level,
                    );

                    if new_child_node.is_empty() {
                        continue;
                    }

                    if new_children.is_empty() {
                        new_children = self.nodes.get_range(node.range()).to_smallvec();
                    }

                    new_children.insert(index_in_children as usize, new_child_node);
                    new_pop_mask |= 1 << i as u64; 

                    continue;
                } 

                let new_child_node = if aabb.contains_aabb(node_aabb) {
                    self.add_pos_query_recursive(
                        model, 
                        lod,
                        gi,
                        min,
                        new_level,
                    )
                } else {
                    self.update_pos_recursive(
                        model,
                        lod,
                        gi,
                        aabb,
                        new_level,
                        min,
                        node.index() + index_in_children,
                    )
                };

                if new_children.is_empty() {
                    new_children = self.nodes.get_range(node.range()).to_smallvec();
                }
                new_children[index_in_children as usize] = new_child_node;
            }           
        }

        if !new_children.is_empty() {

            let index = self.nodes.push(&new_children);
            let gi_index = gi.new_probe_index(offset, level, new_pop_mask, &new_children); 

            VoxelDAG64Node::new(false, index, new_pop_mask, gi_index)
        } else {
            node
        }
    }
}
