use octa_force::{glam::{vec3a, UVec3, Vec3A, Vec4Swizzles}, log::debug, OctaResult};

use crate::{multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, util::aabb::AABB, volume::VolumeQureyAABB};

use super::{node::VoxelDAG64Node, VoxelDAG64};


impl DAG64Transaction {
    pub fn update_aabb<M: VolumeQureyAABB>(
        &mut self, 
        dag: &mut VoxelDAG64,
        changed_aabb: AABB, 
        last_offset: Vec3A, 
        model: &M, 
        allocator: &mut BuddyBufferAllocator
    ) -> OctaResult<()> {
        let mut scale = 4_u32.pow(self.new_levels as u32) as f32;
        let mut min = last_offset;
        let mut max = min + scale;
        let mut tree_aabb = AABB::new_a(min, max);

        // Increase the Tree if the model does not fit.
        let model_aabb= model.get_bounds();
        while !tree_aabb.contains_aabb(model_aabb) {
            debug!("Expand Tree");
            let child_pos = Vec3A::from(tree_aabb.min - model_aabb.min).max(Vec3A::ZERO) % scale;
            let child_index = child_pos.round().as_uvec3().dot(UVec3::new(1, 4, 16));

            let new_root = VoxelDAG64Node::new(false, self.new_root_index, 1 << child_index as u64);
            self.new_root_index = dag.nodes.push(&[new_root], allocator)? as u32;
            
            self.new_levels += 1;
            min = min - child_pos * scale;
            scale = 4_u32.pow(self.new_levels as u32) as f32;
            max = min + scale;
            tree_aabb = AABB::new_a(min, max);
        }

        self.next_node(dag, changed_aabb, model, allocator, self.new_levels, min, self.new_root_index)?;

        Ok(())
    }

    fn next_node<M: VolumeQureyAABB>(
        &mut self, 
        dag: &mut VoxelDAG64, 
        aabb: AABB, 
        model: &M, 
        allocator: &mut BuddyBufferAllocator, 
        node_level: u8, 
        offset: Vec3A, 
        index: u32
    ) -> OctaResult<()> {

        let node = self.get_node(dag, index).unwrap();

        // TODO somehow are some parts missing at level 3
        if node.is_leaf() || node_level <= 4 {
            let new_node = dag.insert_from_aabb_query_recursive(
                model, 
                offset,
                node_level,
                allocator,
            )?;

            if new_node != node {
                self.change_node(index, new_node, node);
            }

            return Ok(());
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

                            let new_child_node = dag.insert_from_aabb_query_recursive(
                                model, 
                                min,
                                node_level -1,
                                allocator,
                            )?;

                            if new_child_node.is_empty() {
                                continue;
                            }

                            if new_children.is_empty() {
                                new_children = self.get_node_range(dag, node.range())?;
                            }

                            new_children.insert(index_in_children as usize, new_child_node);
                            new_bitmask |= 1 << child_nr as u64; 

                            continue;
                        } 

                        if aabb.contains_aabb(node_aabb) {

                            let new_child_node = dag.insert_from_aabb_query_recursive(
                                model, 
                                min,
                                node_level -1,
                                allocator,
                            )?;

                            if new_children.is_empty() {
                                new_children = self.get_node_range(dag, node.range())?;
                            }
                            
                            new_children[index_in_children as usize] = new_child_node;

                            continue;
                        } 

                        self.next_node(
                            dag,
                            aabb,
                            model,
                            allocator,
                            node_level - 1,
                            min,
                            node.ptr() + index_in_children,
                        )?;
                    }                
                }
            }
        }

        if !new_children.is_empty() {
            let new_node = VoxelDAG64Node::new(false, dag.nodes.push(&new_children, allocator)? as u32, new_bitmask);

            if new_node != node {
                self.change_node(index, new_node, node);
            }
        }

        Ok(())
    }
}
