use octa_force::{glam::UVec3, log::debug, OctaResult};


use super::{node::VoxelDAG64Node, VoxelDAG64};

impl VoxelDAG64 {
    pub fn from_pos_query<M: VolumeQureyPosValue>(model: &M, allocator: &mut BuddyBufferAllocator) -> OctaResult<Self> {
        let dims = model.get_size();
        let mut scale = dims[0].max(dims[1]).max(dims[2]).next_power_of_two();
        scale = scale.max(4);
        if scale.ilog2() % 2 == 1 {
            scale *= 2;
        }

        let levels = scale.ilog(4) as _;
        let mut this = Self {
            nodes: CacheAllocatedVec::new(4000 * size_of::<VoxelDAG64Node>()),
            data: CacheAllocatedVec::new(64),
            levels,
            root_index: 0,
        };

        let root = this.insert_from_pos_query_recursive(model, UVec3::ZERO, levels, allocator)?;
        this.root_index = this.nodes.push(&[root], allocator)? as _;

        this.nodes.optimize();
        this.data.optimize();

        Ok(this)
    }
}

impl VoxelDAG64 {
    fn insert_from_pos_query_recursive<M: VolumeQureyPosValue>(
        &mut self,
        model: &M,
        offset: UVec3,
        node_level: u8,
        allocator: &mut BuddyBufferAllocator,
    ) -> OctaResult<VoxelDAG64Node> {
        let mut bitmask = 0;

        if node_level == 1 {
            let mut vec = arrayvec::ArrayVec::<_, 64>::new();
            for z in 0..4 {
                for y in 0..4 {
                    for x in 0..4 {
                        let pos = UVec3::new(x, y, z);
                        let index = offset + pos;
                        let value = model.get_value(index);

                        if value != 0 {
                            vec.push(value);
                            bitmask |= 1 << pos.dot(UVec3::new(1, 4, 16)) as u64;
                        }
                    }
                }
            }

            Ok(VoxelDAG64Node::new(true, self.data.push(&vec, allocator)? as u32, bitmask))
        } else {
            let new_scale = 4_u32.pow(node_level as u32 - 1);
            let mut nodes = arrayvec::ArrayVec::<_, 64>::new();
            for z in 0..4 {
                for y in 0..4 {
                    for x in 0..4 {
                        let pos = UVec3::new(x, y, z);
                        if let Some(child) = self.insert_from_pos_query_recursive(
                                model,
                                offset + pos * new_scale,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use octa_force::glam::UVec3;

    use crate::{multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, voxel_dag64::VoxelDAG64, voxel_grid::VoxelGrid};

    #[test]
    pub fn test() {
        let mut grid = VoxelGrid::new(UVec3::ONE * 4_u32.pow(4)); 
        grid.set_example_sphere();
        grid.set_corners();

        let buffer_size = 2_usize.pow(30);
        let mut allocator = BuddyBufferAllocator::new(buffer_size, 32);
        let tree64: VoxelDAG64 = VoxelDAG64::from_pos_query(&grid, &mut allocator).unwrap();
    }
}
