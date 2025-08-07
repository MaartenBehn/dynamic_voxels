use octa_force::{glam::{IVec3, UVec3, Vec3A}, log::debug, OctaResult};


use crate::{multi_data_buffer::{allocated_vec::AllocatedVec, buddy_buffer_allocator::BuddyBufferAllocator, cached_vec::CachedVec}, volume::{VolumeQureyPosValueI}};

use super::{node::VoxelDAG64Node, DAG64EntryData, DAG64EntryKey, VoxelDAG64};

impl VoxelDAG64 {
    pub fn add_pos_query_volume<M: VolumeQureyPosValueI>(&mut self, model: &M) -> OctaResult<DAG64EntryKey> {
        let offset = model.get_offset_i();
        let dims = model.get_size_i().as_uvec3();
        let mut scale = dims[0].max(dims[1]).max(dims[2]).next_power_of_two();
        scale = scale.max(4);
        if scale.ilog2() % 2 == 1 {
            scale *= 2;
        }

        let levels = scale.ilog(4) as _;
        
        let root = self.add_pos_query_recursive(model, offset, levels)?;
        let root_index = self.nodes.push(&[root])?;
        let key = self.entry_points.insert(DAG64EntryData { 
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
            let mut vec = arrayvec::ArrayVec::<_, 64>::new();
            for z in 0..4 {
                for y in 0..4 {
                    for x in 0..4 {
                        // INFO: DAG Renderer works in XZY Space instead of XYZ like the rest of the
                        // engine
                        let pos = offset + IVec3::new(x, z, y);
                        let value = model.get_value_i(pos);

                        if value != 0 {
                            vec.push(value);
                            bitmask |= 1 << IVec3::new(x, y, z).dot(IVec3::new(1, 4, 16)) as u64;
                        }
                    }
                }
            }

            Ok(VoxelDAG64Node::new(true, self.data.push(&vec)? as u32, bitmask))
        } else {
            let new_scale = 4_i32.pow(node_level as u32 - 1);
            let mut nodes = arrayvec::ArrayVec::<_, 64>::new();
            for z in 0..4 {
                for y in 0..4 {
                    for x in 0..4 {
                        if let Some(child) = self.add_pos_query_recursive(
                                model,
                                offset + IVec3::new(x, z, y) * new_scale,
                                node_level - 1,
                            )?
                            .check_empty()
                        {
                            nodes.push(child);
                            bitmask |= 1 << IVec3::new(x, y, z).dot(IVec3::new(1, 4, 16)) as u64;
                        }
                    }
                }
            }

            Ok(VoxelDAG64Node::new(false, self.nodes.push(&nodes)? as u32, bitmask))
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use octa_force::glam::UVec3;

    use crate::{multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, voxel::{dag64::VoxelDAG64, grid::VoxelGrid}};

    #[test]
    pub fn test() {
        let mut grid = VoxelGrid::new(UVec3::ONE * 4_u32.pow(4)); 
        grid.set_example_sphere();
        grid.set_corners();

        let tree64: VoxelDAG64 = VoxelDAG64::from_pos_query(&grid).unwrap();
    }
}
*/
