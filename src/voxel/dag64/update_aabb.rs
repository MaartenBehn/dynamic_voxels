use octa_force::{glam::{vec3a, UVec3, Vec3A, Vec4Swizzles}, log::debug, OctaResult};

use crate::{multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, util::aabb::AABB, volume::VolumeQureyAABB};

use super::{node::VoxelDAG64Node, DAG64EntryData, DAG64EntryKey, VoxelDAG64};


impl VoxelDAG64 {
    pub fn update_aabb<M: VolumeQureyAABB>(
        &mut self, 
        model: &M,
        changed_aabb: AABB, 
        based_on_entry: DAG64EntryKey,
    ) -> OctaResult<DAG64EntryKey> {
        let mut entry_data = self.entry_points[based_on_entry].to_owned(); 

        let mut scale = 4_u32.pow(entry_data.levels as u32) as f32;
        let mut tree_aabb = AABB::new_a(entry_data.offset, entry_data.offset + scale as f32);


        // Increase the Tree if the model does not fit.
        let model_aabb = model.get_bounds();
        let model_center = model_aabb.center();

        dbg!(entry_data.offset);
        while !tree_aabb.contains_aabb(model_aabb) {
            let child_pos = (Vec3A::from(model_center - tree_aabb.min) / scale) + 1.0;
            let child_index = child_pos.as_uvec3().dot(UVec3::new(1, 4, 16));

            let new_root = VoxelDAG64Node::new(false, entry_data.root_index, 1 << child_index as u64);
            entry_data.root_index = self.nodes.push(&[new_root])? as u32;
            
            entry_data.levels += 1;
            entry_data.offset = entry_data.offset - child_pos * scale as f32;
            scale = 4_u32.pow(entry_data.levels as u32) as f32;
            tree_aabb = AABB::new_a(entry_data.offset, entry_data.offset + scale as f32);
            debug!("Expand Tree {tree_aabb:?}");
            dbg!(entry_data.offset);
        }

        let root = self.next_node(model, changed_aabb,entry_data.levels, entry_data.offset, entry_data.root_index)?;
        entry_data.root_index = self.nodes.push(&[root])?;

        let key = self.entry_points.insert(entry_data);

        Ok(key)
    }

    fn next_node<M: VolumeQureyAABB>(
        &mut self, 
        model: &M, 
        aabb: AABB, 
        node_level: u8, 
        offset: Vec3A, 
        index: u32
    ) -> OctaResult<VoxelDAG64Node> {
        let node = self.nodes.get(index);

        if node.is_leaf() {
            let new_node = self.insert_from_aabb_query_recursive(
                model, 
                offset,
                node_level,
            )?;

            return Ok(new_node);
        }

        let mut new_children = vec![];
        let mut new_bitmask = node.pop_mask;
                
        let new_scale = 4_u32.pow(node_level as u32 - 1) as f32;
        for z in (0..4).rev() {
            for y in (0..4).rev() {
                for x in (0..4).rev() {
                    let pos = UVec3::new(x, y, z);
                    let min = offset + pos.as_vec3a() * new_scale;
                    let max = min + new_scale;
                    let node_aabb = AABB::new_a(min, max);

                    if aabb.collides_aabb(node_aabb) {

                        let child_nr = pos.dot(UVec3::new(1, 4, 16));
                        let index_in_children = node.get_index_in_children_unchecked(child_nr);
                        if !node.is_occupied(child_nr) {

                            let new_child_node = self.insert_from_aabb_query_recursive(
                                model, 
                                min,
                                node_level -1,
                            )?;

                            if new_child_node.is_empty() {
                                continue;
                            }

                            if new_children.is_empty() {
                                new_children = self.nodes.get_range(node.range()).to_vec();
                            }

                            new_children.insert(index_in_children as usize, new_child_node);
                            new_bitmask |= 1 << child_nr as u64; 

                            continue;
                        } 

                        let new_child_node = if aabb.contains_aabb(node_aabb) {
                            self.insert_from_aabb_query_recursive(
                                model, 
                                min,
                                node_level -1,
                            )?
                        } else {
                            self.next_node(
                                model,
                                aabb,
                                node_level - 1,
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
