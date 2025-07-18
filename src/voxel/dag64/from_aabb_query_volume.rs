use octa_force::{glam::{vec3, vec3a, UVec3, Vec3, Vec3A}, log::debug, OctaResult};


use crate::{multi_data_buffer::{allocated_vec::AllocatedVec, buddy_buffer_allocator::BuddyBufferAllocator}, util::aabb::AABB, volume::{VolumeQureyAABB, VolumeQureyAABBResult}};

use super::{node::VoxelDAG64Node, VoxelDAG64};

impl VoxelDAG64 {
    pub fn from_aabb_query<M: VolumeQureyAABB>(model: &M, allocator: &mut BuddyBufferAllocator) -> OctaResult<Self> {
        let offset = model.get_offset();
        let dims = model.get_size().as_uvec3();
        let mut scale = dims[0].max(dims[1]).max(dims[2]).next_power_of_two();
        scale = scale.max(4);
        if scale.ilog2() % 2 == 1 {
            scale *= 2;
        }

        let levels = scale.ilog(4) as _;
        let mut this = Self {
            nodes: AllocatedVec::new(40000 * size_of::<VoxelDAG64Node>()),
            data: AllocatedVec::new(64),
            levels,
            root_index: 0,
            offset,
        };

        let root = this.insert_from_aabb_query_recursive(model, offset, levels, allocator)?;
        this.root_index = this.nodes.push(&[root], allocator)? as _;

        this.nodes.optimize();
        this.data.optimize();

        Ok(this)
    }
}

impl VoxelDAG64 {
    pub fn insert_from_aabb_query_recursive<M: VolumeQureyAABB>(
        &mut self,
        model: &M,
        offset: Vec3A,
        node_level: u8,
        allocator: &mut BuddyBufferAllocator,
    ) -> OctaResult<VoxelDAG64Node> {
        let mut bitmask = 0;

        if node_level == 1 {
            let scale = 4_u32.pow(node_level as u32) as f32;
            let aabb = AABB::new_a(
                offset, 
                offset + vec3a(scale, scale, scale));

            let res = model.get_aabb_value(aabb);

            match res {
                VolumeQureyAABBResult::Full(v) => {
                    if v == 0 {
                        Ok(VoxelDAG64Node::new(true, 0, 0))
                    } else {
                        Ok(VoxelDAG64Node::new(true, self.data.push(&[v; 64], allocator)? as u32, u64::MAX))
                    }
                },
                VolumeQureyAABBResult::Mixed =>  {
                    let new_scale = 4_u32.pow(node_level as u32 - 1) as f32;
                    let mut vec = arrayvec::ArrayVec::<_, 64>::new();
                    for z in 0..4 {
                        for y in 0..4 {
                            for x in 0..4 {
                                let pos = UVec3::new(x, y, z);
                                let index = offset + pos.as_vec3a();
                                let value = model.get_value_a(index);

                                if value != 0 {
                                    vec.push(value);
                                    bitmask |= 1 << pos.dot(UVec3::new(1, 4, 16)) as u64;
                                }
                            }
                        }
                    }

                    Ok(VoxelDAG64Node::new(true, self.data.push(&vec, allocator)? as u32, bitmask))
                },
            }
        } else {
            let scale = 4_u32.pow(node_level as u32) as f32;
            let aabb = AABB::new_a(
                offset, 
                offset + vec3a(scale, scale, scale));

            let res = model.get_aabb_value(aabb); 

            match res {
                VolumeQureyAABBResult::Full(v) => {
                    if v == 0 {
                        Ok(VoxelDAG64Node::new(true, 0, 0))
                    } else {
                        Ok(VoxelDAG64Node::new(true, self.data.push(&[v; 64], allocator)? as u32, u64::MAX))
                    }
                },
                VolumeQureyAABBResult::Mixed =>  {
                    let new_scale = 4_u32.pow(node_level as u32 - 1) as f32;
                    let mut nodes = arrayvec::ArrayVec::<_, 64>::new();
                    for z in 0..4 {
                        for y in 0..4 {
                            for x in 0..4 {
                                let pos = UVec3::new(x, y, z);
                                if let Some(child) = self.insert_from_aabb_query_recursive(
                                    model,
                                    offset + pos.as_vec3a() * new_scale,
                                    node_level - 1,
                                    allocator,
                                )?
                                    .check_empty()
                                {
                                    nodes.push(child);
                                    bitmask |= 1 << pos.dot(UVec3::new(1, 4, 16)) as u64;
                                }
                            }
                        }
                    }

                    Ok(VoxelDAG64Node::new(false, self.nodes.push(&nodes, allocator)? as u32, bitmask))
                },
            }
        }
    }
}


#[cfg(test)]

mod tests {
    use octa_force::glam::Vec3;

    use crate::{csg::{fast_query_csg_tree::tree::FastQueryCSGTree, slot_map_csg_tree::tree::SlotMapCSGTree, vec_csg_tree::tree::VecCSGTree}, multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, voxel::dag64::VoxelDAG64};

    #[test]
    fn build_from_different_csg_trees_should_result_the_same() {
        
        let buffer_size = 2_usize.pow(30);

        let csg_1: FastQueryCSGTree<u8>  = VecCSGTree::new_sphere(Vec3::ZERO, 100.0).into();

        let mut allocator_1 = BuddyBufferAllocator::new(buffer_size, 32);
        let tree64_1 = VoxelDAG64::from_aabb_query(&csg_1, &mut allocator_1).unwrap();

        let csg_2 = SlotMapCSGTree::new_sphere(Vec3::ZERO, 100.0);

        let mut allocator_2 = BuddyBufferAllocator::new(buffer_size, 32);
        let tree64_2 = VoxelDAG64::from_aabb_query(&csg_2, &mut allocator_2).unwrap();

        dbg!(csg_1);
        dbg!(csg_2);

        dbg!(&tree64_1);
        dbg!(&tree64_2);

        assert_eq!(tree64_1, tree64_2);
        assert_eq!(allocator_1, allocator_2);
    }
}
