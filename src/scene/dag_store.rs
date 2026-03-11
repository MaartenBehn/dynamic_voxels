use std::time::Instant;

use octa_force::{OctaResult, log::{debug, info}, vulkan::{Buffer, Context, ash::vk, gpu_allocator::MemoryLocation}};
use slotmap::{new_key_type, SlotMap};
use smallvec::SmallVec;

use crate::{scene::{object::SceneObject, staging_copies::SceneStagingBuilder, worker::SceneObjectKey}, util::buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, voxel::dag64::{lod_heuristic::{LODHeuristicNone, LinearLODHeuristicSphere, PowHeuristicSphere}, parallel::ParallelVoxelDAG64}};

new_key_type! { pub struct SceneDAGKey; }


#[derive(Debug)]
pub struct SceneDAG {
    pub dag: ParallelVoxelDAG64,
    pub node_alloc: ManualBuddyAllocation,
    pub data_alloc: ManualBuddyAllocation,
    pub objects: SmallVec<[SceneObjectKey; 4]>,
    pub needs_update: bool,
    pub check_clean: bool,
}

#[derive(Debug)]
pub struct SceneDAGStore {
    pub dags: SlotMap<SceneDAGKey, SceneDAG>,
    pub needs_update: bool,
    pub check_clean: bool,
}

impl SceneDAGStore {
    pub fn new() -> Self {
        Self {
            dags: SlotMap::default(),
            needs_update: false,
            check_clean: false,
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
            objects: SmallVec::new(),
            needs_update: false,
            check_clean: false,
        }))
    }

    pub fn mark_changed(&mut self, key: SceneDAGKey) {
        self.dags[key].needs_update = true;
        self.dags[key].check_clean = true;
        self.needs_update = true;
        self.check_clean = true;
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
        if !self.needs_update {
            return;
        }

        for dag in self.dags.values_mut() {
            if dag.needs_update {
                dag.dag.nodes.push_scene_builder(builder, dag.node_alloc.start());
                dag.dag.data.push_scene_builder(builder, dag.data_alloc.start());
                dag.needs_update = false;
            }
        }
        self.needs_update = false;
    }

    pub fn clean(&mut self, objects: &mut SlotMap<SceneObjectKey, SceneObject>) {
        if !self.check_clean {
            return;
        }

        let max_filled = 0.8;
        for dag in self.dags.values_mut() {
            if dag.check_clean {
                if dag.dag.is_filled_to(max_filled) {
                    let now = Instant::now();

                    dag.dag.clean();

                    let elapsed = now.elapsed();
                    debug!("DAG Clean took: {:?}", elapsed);

                    for object_key in dag.objects.iter() {
                        let object = objects[*object_key].get_dag_object_mut();
                        object.entry = dag.dag.get_entry(object.entry_key);
                    }
                }

                dag.check_clean = false;
            }
        }
        self.check_clean = false;
    }
}
