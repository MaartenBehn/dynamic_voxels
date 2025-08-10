pub mod add_pos_query_volume;
pub mod add_aabb_query_volume;

use std::sync::Arc;

use parking_lot::Mutex;
use slotmap::SlotMap;

use crate::multi_data_buffer::{parallel_vec::ParallelVec};

use super::{node::VoxelDAG64Node, DAG64EntryData, DAG64EntryKey, VoxelDAG64};

#[derive(Debug, Clone)]
pub struct ParallelVoxelDAG64 {
    pub nodes: ParallelVec<VoxelDAG64Node>,
    pub data: ParallelVec<u8>,
    pub entry_points: Arc<Mutex<SlotMap<DAG64EntryKey, DAG64EntryData>>>
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
}
