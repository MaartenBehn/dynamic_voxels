// based on https://github.com/expenses/tree64 

pub mod add_pos_query_volume;
pub mod add_aabb_query_volume;
pub mod update_aabb_query_volume;
pub mod update_pos_query_volume;
pub mod expand;

use octa_force::{OctaResult, glam::{IVec3, Mat4, Quat, Vec3, Vec3A}, log::{debug, info}};
use slotmap::{new_key_type, SlotMap};

use crate::{VOXELS_PER_SHADER_UNIT, util::{math::to_mb, number::Nu, reuse_buffer::ReUseBuffer, vector::Ve}, volume::{VolumeQureyAABB, VolumeQureyPosValue}, voxel::dag64::{entry::{DAG64Entry, DAG64EntryKey}, lod_heuristic::{LODHeuristicNone, LODHeuristicT}, node::VoxelDAG64Node}};


#[derive(Debug)]
pub struct VoxelDAG64 {
    pub nodes: ReUseBuffer<VoxelDAG64Node>,
    pub data: ReUseBuffer<u8>,
    pub entry_points: SlotMap<DAG64EntryKey, DAG64Entry>,
}

impl VoxelDAG64 { 
    pub fn new(nodes_capacity: usize, data_capacity: usize) -> Self {
        Self {
            nodes: ReUseBuffer::new(nodes_capacity),
            data: ReUseBuffer::new(data_capacity),
            entry_points: Default::default(),
        }
    }

    pub fn get_first_key(&self) -> DAG64EntryKey {
        self.entry_points.keys().next().unwrap().to_owned()
    }

     fn print_memory_info(&self) {
        info!("VoxelDAG64: nodes {} MB, data {} MB", 
            to_mb(self.nodes.get_memory_size()),
            to_mb(self.data.get_memory_size()),
        );
    }

    pub fn get_entry(&self, key: DAG64EntryKey) -> DAG64Entry {
        self.entry_points[key].to_owned()
    }

    pub fn remove_entry(&mut self, key: DAG64EntryKey) {
        self.entry_points.remove(key);
    }

    pub(super) fn empty_entry(&mut self) -> OctaResult<DAG64EntryKey> {

        let root_index = self.nodes.push(&[VoxelDAG64Node::single(true, 0, 0)])?;
        let key = self.entry_points.insert(DAG64Entry { 
            levels: 1, 
            root_index, 
            offset: IVec3::ZERO, 
        });

        Ok(key)
    }
}

impl PartialEq for VoxelDAG64 {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes 
        && self.data == other.data 
    }
}
