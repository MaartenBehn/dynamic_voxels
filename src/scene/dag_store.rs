use octa_force::{vulkan::{ash::vk, gpu_allocator::MemoryLocation, AllocContext, Buffer, Context}, OctaResult};
use slotmap::{new_key_type, SlotMap};

use crate::voxel::dag64::parallel::ParallelVoxelDAG64;

new_key_type! { pub struct SceneDAGKey; }

#[derive(Debug)]
pub struct SceneDAG {
    pub dag: ParallelVoxelDAG64,
    pub changed: bool,
    pub node_buffer: Buffer,
    pub data_buffer: Buffer,
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

    pub fn add_dag(&mut self, context: &AllocContext, dag: ParallelVoxelDAG64) -> OctaResult<SceneDAGKey> {
        let mut node_buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            MemoryLocation::CpuToGpu,
            dag.nodes.get_memory_size() as _,
        )?;

        let mut data_buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            MemoryLocation::CpuToGpu,
            dag.data.get_memory_size() as _,
        )?;

        Ok(self.dags.insert(SceneDAG {
            dag,
            changed: true,
            node_buffer, 
            data_buffer,
        }))
    }

    pub fn mark_changed(&mut self, key: SceneDAGKey) {
        self.dags[key].changed = true;
    }

    pub fn get_dag(&self, key: SceneDAGKey) -> &ParallelVoxelDAG64 {
        &self.dags[key].dag
    }

    pub fn flush(&mut self) {
        for scene_dag in self.dags.values_mut() {
            if scene_dag.changed {
                scene_dag.dag.nodes.flush(&scene_dag.node_buffer);
                scene_dag.dag.data.flush(&scene_dag.data_buffer);
                scene_dag.changed = false;
            }
        }
    }
}
