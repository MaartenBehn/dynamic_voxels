pub mod add_pos_query_volume;
pub mod add_aabb_query_volume;
pub mod update_pos_query_volume;
pub mod update_aabb_query_volume;
pub mod expand;
pub mod clean;

use std::sync::Arc;

use octa_force::log::info;
use parking_lot::Mutex;
use slotmap::SlotMap;

use crate::{util::math::to_mb, voxel::{dag64::lod_heuristic::LODHeuristicT, reuse_buffer::parallel_vec::ParallelVec}};

use super::{node::VoxelDAG64Node, DAG64Entry, DAG64EntryKey, VoxelDAG64};

pub const MIN_PAR_LEVEL: u8 = 3;

#[derive(Debug, Clone)]
pub struct ParallelVoxelDAG64 {
    pub nodes: ParallelVec<VoxelDAG64Node>,
    pub inactive_nodes: ParallelVec<VoxelDAG64Node>,
    pub data: ParallelVec<u8>,
    pub inactive_data: ParallelVec<u8>,
    pub entry_points: Arc<Mutex<SlotMap<DAG64EntryKey, DAG64Entry>>>,
}

impl ParallelVoxelDAG64 {
    pub fn new(nodes_capacity: usize, data_capacity: usize) -> Self {
        Self {
            nodes: ParallelVec::new(nodes_capacity),
            inactive_nodes: ParallelVec::new(nodes_capacity),
            data: ParallelVec::new(data_capacity),
            inactive_data: ParallelVec::new(data_capacity),
            entry_points: Default::default(),
        }
    }

    pub fn print_memory_info(&self) { 
        info!("VoxelDAG64: nodes {} MB {}%, data {} MB {}%", 
            to_mb(self.nodes.get_memory_size()),
            self.nodes.filled() * 100.0,
            to_mb(self.data.get_memory_size()),
            self.data.filled() * 100.0,
        );
    }

    pub fn get_entry(&self, key: DAG64EntryKey) -> DAG64Entry {
        self.entry_points.lock()[key].to_owned()
    }

    pub fn remove_entry(&mut self, key: DAG64EntryKey) {
        self.entry_points.lock().remove(key);
    }

    pub fn is_filled_to(&self, factor: f32) -> bool {
        //dbg!(self.nodes.filled());
        //dbg!(self.data.filled());
        self.nodes.filled() > factor || self.data.filled() > factor
    }
}
