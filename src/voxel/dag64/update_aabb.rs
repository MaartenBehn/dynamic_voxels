use octa_force::{glam::{vec3a, UVec3, Vec3A, Vec4Swizzles}, OctaResult};

use crate::{multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, util::aabb::AABB, volume::VolumeQureyAABB};

use super::{changes::DAG64Transaction, node::VoxelDAG64Node, VoxelDAG64};



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

        self.next_node(changed_aabb, model, allocator, self.new_levels, min);

        Ok(())
    }

    fn next_node<M: VolumeQureyAABB>(&mut self, aabb: AABB, model: &M, allocator: &mut BuddyBufferAllocator, node_level: u8, offset: Vec3A) {

        if node_level == 1 {
            // Found
        }
                
        let new_scale = 4_u32.pow(node_level as u32 - 1) as f32;
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..4 {
                    let pos = UVec3::new(x, y, z);
                    let min = offset + pos.as_vec3a() * new_scale;
                    let max = min + new_scale;
                    let node_aabb = AABB::new_a(min, max);
                    if aabb.collides_aabb(node_aabb) {
                        if aabb.contains_aabb(node_aabb) {
                            // Found
                        } else {
                            self.next_node(
                                aabb,
                                model,
                                allocator,
                                node_level - 1,
                                offset + pos.as_vec3a() * new_scale,
                            );
                        }
                    }
                }
            }
        }

    }
}
