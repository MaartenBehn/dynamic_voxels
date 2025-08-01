// based on https://github.com/expenses/tree64 

pub mod node;
pub mod add_pos_query_volume;
pub mod add_aabb_query_volume;
pub mod update_aabb;

use node::VoxelDAG64Node;
use octa_force::{glam::{Vec3, Vec3A}, log::{debug, info}};
use slotmap::{new_key_type, SlotMap};

use crate::{multi_data_buffer::{allocated_vec::AllocatedVec, cached_vec::CachedVec}, util::math::to_mb};

new_key_type! { pub struct DAG64EntryKey; }

#[derive(Debug)]
pub struct VoxelDAG64 {
    pub nodes: CachedVec<VoxelDAG64Node>,
    pub data: CachedVec<u8>,
    pub entry_points: SlotMap<DAG64EntryKey, DAG64EntryData>
}

#[derive(Debug, Clone, Copy)]
pub struct DAG64EntryData {
    pub levels: u8,
    pub root_index: u32,
    pub offset: Vec3A,
}

impl VoxelDAG64 { 
    pub fn new(nodes_capacity: usize, data_capacity: usize) -> Self {
        Self {
            nodes: CachedVec::new(nodes_capacity),
            data: CachedVec::new(data_capacity),
            entry_points: Default::default(),
        }
    }

    pub fn print_memory_info(&self) { 
        info!("VoxelDAG64: nodes {} MB, data {} MB", 
            to_mb(self.nodes.get_memory_size()),
            to_mb(self.data.get_memory_size()),
        );
    }

    pub fn get_first_key(&self) -> DAG64EntryKey {
        self.entry_points.keys().next().unwrap().to_owned()
    }
}

impl DAG64EntryData { 
    pub fn get_size(&self) -> u32 {
        4_u32.pow(self.levels as u32)
    }
}


impl PartialEq for VoxelDAG64 {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes 
        && self.data == other.data 
    }
}
