pub mod add_pos_query_volume;

use std::sync::Arc;

use parking_lot::Mutex;
use slotmap::SlotMap;

use crate::multi_data_buffer::worker_vec::WorkerVec;

use super::{node::VoxelDAG64Node, DAG64EntryData, DAG64EntryKey, VoxelDAG64};

pub struct ParallelVoxelDAG64 {
    pub nodes: WorkerVec<64, VoxelDAG64Node>,
    pub data: WorkerVec<64, u8>,
    pub entry_points: Arc<Mutex<SlotMap<DAG64EntryKey, DAG64EntryData>>>
}

impl VoxelDAG64 {
    pub fn run_worker(self, channel_cap: usize) -> ParallelVoxelDAG64 {

        let nodes = self.nodes.run_worker(channel_cap);
        let data = self.data.run_worker(channel_cap);
        let entry_points = Arc::new(Mutex::new(self.entry_points));

        ParallelVoxelDAG64 {
            nodes,
            data,
            entry_points,
        }
    }
}

impl ParallelVoxelDAG64 {
    pub fn stop(self) -> VoxelDAG64 {
        let nodes = self.nodes.stop();
        let data = self.data.stop();
        let entry_points = Arc::try_unwrap(self.entry_points).unwrap().into_inner();

        VoxelDAG64 { nodes, data, entry_points }
    }
}
