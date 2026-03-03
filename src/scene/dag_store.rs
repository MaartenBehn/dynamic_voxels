use octa_force::{vulkan::{ash::vk, gpu_allocator::MemoryLocation, Context, Buffer}, OctaResult};
use slotmap::{new_key_type, SlotMap};

use crate::{util::buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, voxel::dag64::{lod_heuristic::{LODHeuristicNone, LinearLODHeuristicSphere, PowHeuristicSphere}, parallel::ParallelVoxelDAG64}};

new_key_type! { pub struct SceneDAGKey; }


#[derive(Debug)]
pub struct SceneDAG {
    pub dag: ParallelVoxelDAG64,
    pub node_alloc: ManualBuddyAllocation,
    pub data_alloc: ManualBuddyAllocation,
    pub changed: bool,
}

#[derive(Debug)]
pub struct SceneDAGStore {
    pub dags: SlotMap<SceneDAGKey, SceneDAG>,
}

impl SceneDAGStore {
    pub fn new() -> Self {
        Self {
            dags: SlotMap::default(),
        }
    }   

    pub fn add_dag(&mut self, dag: ParallelVoxelDAG64, allocator: &mut BuddyAllocator) -> OctaResult<SceneDAGKey> {

        let node_alloc = allocator.alloc(dag.nodes.get_memory_size())?;
        let data_alloc = allocator.alloc(dag.data.get_memory_size())?;

        Ok(self.dags.insert(SceneDAG {
            dag,
            node_alloc,
            data_alloc,
            changed: true,
        }))
    }

    pub fn mark_changed(&mut self, key: SceneDAGKey) {
        self.dags[key].changed = true;
    }

    pub fn get_dag(&self, key: SceneDAGKey) -> &ParallelVoxelDAG64 {
        &self.dags[key].dag
    }
    
    pub fn remove_dag(&mut self, key: SceneDAGKey, allocator: &mut BuddyAllocator) {
        if let Some(d) = self.dags.remove(key) {
            allocator.dealloc(d.node_alloc);
            allocator.dealloc(d.data_alloc);
        }
    }
}
