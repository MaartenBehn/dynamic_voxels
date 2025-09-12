use octa_force::{glam::{vec3, vec3a, IVec3, UVec3, Vec3, Vec3A}, log::debug, OctaResult};
use smallvec::SmallVec;


use crate::{multi_data_buffer::{buddy_buffer_allocator::BuddyBufferAllocator, cached_vec::CachedVec}, util::{aabb::AABB, math::get_dag_node_children_xzy_i, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeQureyAABB, VolumeQureyAABBResult}};

use super::{node::VoxelDAG64Node, util::get_dag_offset_levels, DAG64Entry, DAG64EntryKey, VoxelDAG64};

impl VoxelDAG64 {
    pub fn add_aabb_query_volume<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3>>(&mut self, model: &M) -> OctaResult<DAG64EntryKey> { 
        let (offset, levels) = get_dag_offset_levels(model);
        if levels == 0 {
            return self.empty_entry();
        }
        
        let root = self.add_aabb_query_recursive(model, offset, levels)?;
        let root_index = self.nodes.push(&[root])?;
        let key = self.entry_points.insert(DAG64Entry { 
            levels, 
            root_index, 
            offset, 
        });
 
        Ok(key)
    }

    pub fn add_aabb_query_recursive<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3>>(
        &mut self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> OctaResult<VoxelDAG64Node> {
        let mut bitmask = 0;

        if node_level == 1 {
            let scale = 4_i32.pow(node_level as u32);
            let aabb = AABB::new(
                V::from_ivec3(offset), 
                V::from_ivec3(offset + scale));

            let res = model.get_aabb_value(aabb);

            match res {
                VolumeQureyAABBResult::Full(v) => {
                    if v == 0 {
                        Ok(VoxelDAG64Node::new(true, 0, 0))
                    } else {
                        Ok(VoxelDAG64Node::new(true, self.data.push(&[v; 64])? as u32, u64::MAX))
                    }
                },
                VolumeQureyAABBResult::Mixed =>  {
                    let mut vec = SmallVec::<[_; 64]>::new();

                    // INFO: DAG Renderer works in XZY Space instead of XYZ like the rest of the
                    // engine
                    for (i, pos) in get_dag_node_children_xzy_i().into_iter().enumerate() {
                        let pos = offset + pos;
                        let value = model.get_value(V::from_ivec3(pos));

                        if value != 0 {
                            vec.push(value);
                            bitmask |= 1 << i as u64;
                        }
                    } 

                    let ptr = self.data.push(&vec)?;
                    Ok(VoxelDAG64Node::new(true, ptr, bitmask))
                },
            }
        } else {
            let scale = 4_i32.pow(node_level as u32);
            let aabb = AABB::new(
                V::from_ivec3(offset), 
                V::from_ivec3(offset + scale));


            let res = model.get_aabb_value(aabb); 

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
}
