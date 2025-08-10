use octa_force::{glam::{vec3, vec3a, IVec3, UVec3, Vec3, Vec3A}, log::debug, OctaResult};
use smallvec::SmallVec;


use crate::{multi_data_buffer::{allocated_vec::AllocatedVec, buddy_buffer_allocator::BuddyBufferAllocator, cached_vec::CachedVec}, util::{aabb3d::AABB, iaabb3d::AABBI, math::get_dag_node_children_xzy_i}, volume::{VolumeQureyAABB, VolumeQureyAABBI, VolumeQureyAABBResult}};

use super::{node::VoxelDAG64Node, util::get_dag_offset_levels, DAG64EntryData, DAG64EntryKey, VoxelDAG64};

impl VoxelDAG64 {
    pub fn add_aabb_query_volume<M: VolumeQureyAABBI>(&mut self, model: &M) -> OctaResult<DAG64EntryKey> { 
        let (offset, levels) = get_dag_offset_levels(model);
        
        let root = self.add_aabb_query_recursive(model, offset, levels)?;
        let root_index = self.nodes.push(&[root])?;
        let key = self.entry_points.insert(DAG64EntryData { 
            levels, 
            root_index, 
            offset, 
        });
 
        Ok(key)
    }

    pub fn add_aabb_query_recursive<M: VolumeQureyAABBI>(
        &mut self,
        model: &M,
        offset: IVec3,
        node_level: u8,
    ) -> OctaResult<VoxelDAG64Node> {
        let mut bitmask = 0;

        if node_level == 1 {
            let scale = 4_i32.pow(node_level as u32);
            let aabb = AABBI::new(
                offset, 
                offset + scale);

            let res = model.get_aabb_value_i(aabb);

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
                        let value = model.get_value_i(pos);

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
            let aabb = AABBI::new(
                offset, 
                offset + scale);

            let res = model.get_aabb_value_i(aabb); 

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


/*
#[cfg(test)]

mod tests {
    use octa_force::glam::Vec3;

    use crate::{csg::{fast_query_csg_tree::tree::FastQueryCSGTree, slot_map_csg_tree::tree::SlotMapCSGTree, vec_csg_tree::tree::VecCSGTree}, multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, voxel::dag64::VoxelDAG64};

    #[test]
    fn build_from_different_csg_trees_should_result_the_same() {
        
        let buffer_size = 2_usize.pow(30);

        let csg_1: FastQueryCSGTree<u8>  = VecCSGTree::new_sphere(Vec3::ZERO, 100.0).into();
        let tree64_1 = VoxelDAG64::from_aabb_query(&csg_1).unwrap();

        let csg_2 = SlotMapCSGTree::new_sphere(Vec3::ZERO, 100.0);
        let tree64_2 = VoxelDAG64::from_aabb_query(&csg_2).unwrap();

        assert_eq!(tree64_1, tree64_2);
    }
}
*/
