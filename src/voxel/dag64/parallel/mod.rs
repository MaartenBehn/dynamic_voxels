pub mod add_pos_query_volume;
pub mod add_aabb_query_volume;
pub mod update_pos_query_volume;
pub mod update_aabb_query_volume;
pub mod expand;

use std::sync::Arc;

use octa_force::log::info;
use parking_lot::Mutex;
use slotmap::SlotMap;

use crate::{multi_data_buffer::parallel_vec::ParallelVec, util::math::to_mb};

use super::{node::VoxelDAG64Node, DAG64Entry, DAG64EntryKey, VoxelDAG64};

#[derive(Debug, Clone)]
pub struct ParallelVoxelDAG64 {
    pub nodes: ParallelVec<VoxelDAG64Node>,
    pub data: ParallelVec<u8>,
    pub entry_points: Arc<Mutex<SlotMap<DAG64EntryKey, DAG64Entry>>>
}

impl VoxelDAG64 {
    pub fn parallel(self) -> ParallelVoxelDAG64 {

        let nodes = self.nodes.parallel();
        let data = self.data.parallel();
        let entry_points = Arc::new(Mutex::new(self.entry_points));

        ParallelVoxelDAG64 {
            nodes,
            data,
            entry_points,
        }
    }
}

impl ParallelVoxelDAG64 {
    pub fn single(self) -> VoxelDAG64 {
        let nodes = self.nodes.single();
        let data = self.data.single();
        let entry_points = Arc::try_unwrap(self.entry_points).unwrap().into_inner();

        VoxelDAG64 { nodes, data, entry_points }
    }

    pub fn print_memory_info(&self) { 
        info!("VoxelDAG64: nodes {} MB, data {} MB", 
            to_mb(self.nodes.get_memory_size()),
            to_mb(self.data.get_memory_size()),
        );
    }

    pub fn get_entry(&self, key: DAG64EntryKey) -> DAG64Entry {
        self.entry_points.lock()[key].to_owned()
    }
}
