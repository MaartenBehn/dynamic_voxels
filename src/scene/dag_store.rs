use octa_force::{OctaResult, log::debug, vulkan::{Buffer, Context, ash::vk, gpu_allocator::MemoryLocation}};
use slotmap::{new_key_type, SlotMap};

use crate::{scene::staging_copies::SceneStagingBuilder, util::buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, voxel::dag64::{lod_heuristic::{LODHeuristicNone, LinearLODHeuristicSphere, PowHeuristicSphere}, parallel::ParallelVoxelDAG64}};

new_key_type! { pub struct SceneDAGKey; }


#[derive(Debug)]
pub struct SceneDAG {
    pub dag: ParallelVoxelDAG64,
    pub node_alloc: ManualBuddyAllocation,
    pub data_alloc: ManualBuddyAllocation,
    pub needs_update: bool,
}

#[derive(Debug)]
pub struct SceneDAGStore {
    pub dags: SlotMap<SceneDAGKey, SceneDAG>,
    pub needs_update: bool,
}

impl SceneDAGStore {
    pub fn new() -> Self {
        Self {
            dags: SlotMap::default(),
            needs_update: false,
        }
    }   

    pub fn add_dag(&mut self, dag: ParallelVoxelDAG64, allocator: &mut BuddyAllocator) -> OctaResult<SceneDAGKey> {

        let node_alloc = allocator.alloc(dag.nodes.get_memory_size())?;
        let data_alloc = allocator.alloc(dag.data.get_memory_size())?;

        self.needs_update = true;
        Ok(self.dags.insert(SceneDAG {
            dag,
            node_alloc,
            data_alloc,
            needs_update: true,
        }))
    }

    pub fn mark_changed(&mut self, key: SceneDAGKey) {
        self.dags[key].needs_update = true;
        self.needs_update = true;
    }

    pub fn active_dag(&self) -> SceneDAGKey {
        self.dags.keys().next().unwrap()
    }

    pub fn get_dag(&self, key: SceneDAGKey) -> &ParallelVoxelDAG64 {
        &self.dags[key].dag
    }

    pub fn get_dag_mut(&mut self, key: SceneDAGKey) -> &mut ParallelVoxelDAG64 {
        &mut self.dags[key].dag
    }
    
    pub fn remove_dag(&mut self, key: SceneDAGKey, allocator: &mut BuddyAllocator) -> OctaResult<()> {
        if let Some(d) = self.dags.remove(key) {
            allocator.dealloc(d.node_alloc)?;
            allocator.dealloc(d.data_alloc)?;
        }

        Ok(())
    }

    pub fn update(&mut self, builder: &mut SceneStagingBuilder) {
        for dag in self.dags.values_mut() {
            if dag.needs_update {
                dag.dag.nodes.push_scene_builder(builder, dag.node_alloc.start());
                dag.dag.data.push_scene_builder(builder, dag.data_alloc.start());
                dag.needs_update = false;
            }
        }
        self.needs_update = false;
    }
}
