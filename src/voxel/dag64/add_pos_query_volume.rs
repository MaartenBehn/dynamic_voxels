use octa_force::{glam::{IVec3, UVec3, Vec3A}, log::debug, OctaResult};
use smallvec::SmallVec;


use crate::{multi_data_buffer::{buddy_buffer_allocator::BuddyBufferAllocator, cached_vec::CachedVec}, util::math::get_dag_node_children_xzy_i, volume::VolumeQureyPosValueI};

use super::{node::VoxelDAG64Node, util::get_dag_offset_levels, DAG64Entry, DAG64EntryKey, VoxelDAG64};

impl VoxelDAG64 {
    pub fn add_pos_query_volume<M: VolumeQureyPosValueI>(&mut self, model: &M) -> OctaResult<DAG64EntryKey> {
        let (offset, levels) = get_dag_offset_levels(model);

        let root = self.add_pos_query_recursive(model, offset, levels)?;
        let root_index = self.nodes.push(&[root])?;
        let key = self.entry_points.insert(DAG64Entry { 
            levels, 
            root_index, 
            offset, 
        });

        Ok(key)
    }

    pub fn add_pos_query_recursive<M: VolumeQureyPosValueI>(
        &mut self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> OctaResult<VoxelDAG64Node> {
        let mut bitmask = 0;

        if node_level == 1 {
            let mut vec = SmallVec::<[_; 64]>::new();

            // INFO: DAG Renderer works in XZY Space instead of XYZ like the rest of the
            // engine
            for (i, pos) in get_dag_node_children_xzy_i().into_iter().enumerate() {
                let pos = offset + pos;
                let value = model.get_value_i(pos);

                if value != 0 {
                    vec.push(value);
                    bitmask |= 1 << i as u64;
                }
            } 

            let ptr = self.data.push(&vec)?;
            Ok(VoxelDAG64Node::new(true, ptr, bitmask))
        } else {
            let new_level = node_level -1;
            let new_scale = 4_i32.pow(new_level as u32);
            let mut nodes = SmallVec::<[_; 64]>::new();

            for (i, pos) in get_dag_node_children_xzy_i().into_iter().enumerate() {
                let child = self.add_pos_query_recursive(
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
        }
    }
}
