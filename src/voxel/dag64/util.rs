use octa_force::{glam::{IVec3, UVec3, Vec3A}, OctaResult};
use crate::{util::{math_config::MC, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyAABB}};

use super::{node::VoxelDAG64Node, parallel::ParallelVoxelDAG64, DAG64Entry, DAG64EntryKey, VoxelDAG64};

pub fn get_dag_offset_levels<V: Ve<T, 3>, T: Nu, M: VolumeBounds<V, T, 3>>(model: &M) -> (IVec3, u8) {
    let offset = model.get_offset().to_ivec3();
    let dims = model.get_size().to_uvec3();
    if dims == UVec3::ZERO {
        return (IVec3::ZERO, 0);
    }

    let mut scale = dims[0].max(dims[1]).max(dims[2]).next_power_of_two();
    scale = scale.max(4);
    if scale.ilog2() % 2 == 1 {
        scale *= 2;
    }

    let levels = scale.ilog(4) as _;

    (offset, levels)
}

impl VoxelDAG64 { 
    pub fn empty_entry(&mut self) -> OctaResult<DAG64EntryKey> {

        let root_index = self.nodes.push(&[VoxelDAG64Node::new(true, 0, 0)])?;
        let key = self.entry_points.insert(DAG64Entry { 
            levels: 1, 
            root_index, 
            offset: IVec3::ZERO, 
        });

        Ok(key)
    }
}

impl ParallelVoxelDAG64 { 
    pub fn empty_entry(&mut self) -> OctaResult<DAG64EntryKey> {

        let root_index = self.nodes.push(&[VoxelDAG64Node::new(true, 0, 0)])?;
        let key = self.entry_points.lock().insert(DAG64Entry { 
            levels: 1, 
            root_index, 
            offset: IVec3::ZERO, 
        });

        Ok(key)
    }
}



