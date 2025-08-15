use octa_force::{glam::{vec3a, IVec3, UVec3, Vec3A, Vec4Swizzles}, log::debug, OctaResult};

use crate::{multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, util::{aabb::AABB, aabb3d::AABB3, iaabb3d::AABBI, math::get_dag_node_children_xzy_i, math_config::MC, vector::Ve}, volume::VolumeQureyAABB};


use super::{node::VoxelDAG64Node, DAG64Entry, DAG64EntryKey, VoxelDAG64};


impl VoxelDAG64 {
    pub fn update_aabb_query_volume<C: MC<3>, M: VolumeQureyAABB<C, 3>>(
        &mut self, 
        model: &M,
        based_on_entry: DAG64EntryKey,
    ) -> OctaResult<DAG64EntryKey> {
        let model_aabb = model.get_bounds();
        let mut entry_data = self.expand_to_include_aabb(based_on_entry, model_aabb)?;

        let root = self.update_aabb_recursive(model, model_aabb,entry_data.levels, entry_data.offset, entry_data.root_index)?;
        entry_data.root_index = self.nodes.push(&[root])?;

        let key = self.entry_points.insert(entry_data);

        Ok(key)
    }

    fn update_aabb_recursive<C: MC<3>, M: VolumeQureyAABB<C, 3>>(
        &mut self, 
        model: &M, 
        aabb: AABB<C, 3>, 
        node_level: u8, 
        offset: IVec3, 
        index: u32
    ) -> OctaResult<VoxelDAG64Node> {
        let node = self.nodes.get(index);

        if node.is_leaf() {
            let new_node = self.add_aabb_query_recursive(
                model, 
                offset,
                node_level,
            )?;

            return Ok(new_node);
        }

        let mut new_children = vec![];
        let mut new_bitmask = node.pop_mask;

        let new_level = node_level - 1;
        let new_scale = 4_i32.pow(new_level as u32);
        for (i, pos) in get_dag_node_children_xzy_i().into_iter()
            .enumerate()
            .rev() {
            let min = offset + pos * new_scale;
            let max = min + new_scale;
            let node_aabb = AABB::new(C::Vector::from_ivec3(min), C::Vector::from_ivec3(max));

            if aabb.collides_aabb(&node_aabb) {

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
                        new_children = self.nodes.get_range(node.range()).to_vec();
                    }

                    new_children.insert(index_in_children as usize, new_child_node);
                    new_bitmask |= 1 << i as u64; 

                    continue;
                } 

                let new_child_node = if aabb.contains_aabb(&node_aabb) {
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
                    new_children = self.nodes.get_range(node.range()).to_vec();
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
