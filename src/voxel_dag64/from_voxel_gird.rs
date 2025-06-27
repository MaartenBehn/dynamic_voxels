use octa_force::glam::UVec3;

use crate::{volume::VolumeQureyPosValue, voxel_grid::VoxelGrid};

use super::{node::VoxelDAG64Node, VoxelDAG64};

impl From<VoxelGrid> for VoxelDAG64 {
    fn from(value: VoxelGrid) -> Self {
        
    }
}

impl VoxelDAG64 {
    fn insert_from_pos_query_recursive<M: VolumeQureyPosValue>(
        &mut self,
        model: &M,
        offset: UVec3,
        node_level: u8,
    ) -> VoxelDAG64Node {
        let mut bitmask = 0;

        if node_level == 1 {
            let mut vec = arrayvec::ArrayVec::<_, 64>::new();
            for z in 0..4 {
                for y in 0..4 {
                    for x in 0..4 {
                        let pos = UVec3::new(x, y, z);
                        let index = offset + pos;
                        if let Some(value) = model.get_value(pos) {
                            vec.push(value);
                            bitmask |= 1 << pos.dot(UVec3::new(1, 4, 16)) as u64;
                        }
                    }
                }
            }

            VoxelDAG64Node::new(true, self.insert_values(&vec), bitmask)
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
                            )
                            .check_empty()
                        {
                            nodes.push(child);
                            bitmask |= 1 << pos.dot(UVec3::new(1, 4, 16)) as u64;
                        }
                    }
                }
            }

            VoxelDAG64Node::new(false, self.insert_nodes(&nodes), bitmask)
        }
    }
}
