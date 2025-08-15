
use std::{rc::Rc, sync::Arc};

use octa_force::{anyhow::anyhow, glam::{vec3, vec4, Mat4, Quat, Vec3, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles}, log::debug, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{multi_data_buffer::buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, util::aabb3d::AABB3, voxel::dag64::{parallel::ParallelVoxelDAG64, DAG64Entry, DAG64EntryKey, VoxelDAG64}, VOXELS_PER_METER, VOXELS_PER_SHADER_UNIT};

use super::{dag_store::{SceneDAG, SceneDAGKey, SceneDAGStore}, Scene, SceneObjectData, SceneObjectKey, SceneObjectType};

#[derive(Debug)]
pub struct SceneDAGObject {
    pub allocation: BuddyAllocation,
    pub mat: Mat4,
    pub dag_key: SceneDAGKey,
    pub entry: DAG64Entry,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SceneDAGObjectData {
    pub mat: Mat4,
    pub inv_mat: Mat4,
    pub node_ptr: u64,
    pub data_ptr: u64,
    pub root_index: u32,
}

#[derive(Debug)]
pub struct SceneAddDAGObject {
    pub mat: Mat4,
    pub dag_key: SceneDAGKey,
    pub entry: DAG64Entry,
}

#[derive(Debug)]
pub struct SceneSetDAGEntry {
    pub object: SceneObjectKey,
    pub entry: DAG64Entry,
}

impl Scene {
    pub fn add_dag64_object(
        &mut self,
        add_dag_object: SceneAddDAGObject,
    ) -> OctaResult<SceneObjectKey> {
        let allocation = self.allocator.alloc(size_of::<SceneDAGObjectData>())?;
        let key = self.add_object(SceneObjectType::DAG64(SceneDAGObject {
            allocation,
            mat: add_dag_object.mat,
            dag_key: add_dag_object.dag_key,
            entry: add_dag_object.entry,
        }));
        self.dag_store.mark_changed(add_dag_object.dag_key);

        Ok(key)
    }

    pub fn set_dag64_entry(&mut self, set: SceneSetDAGEntry) -> OctaResult<()> {
        let object = self.objects.get_mut(set.object)
            .ok_or_else(|| anyhow!("Invalid SceneObjectKey!"))?;

        match &mut object.data {
            SceneObjectType::DAG64(dag) => {
                dag.entry = set.entry;
                self.dag_store.mark_changed(dag.dag_key);
            },
        }
        object.changed = true;

        Ok(())
    }
}

impl SceneDAGObject { 
    pub fn push_to_buffer(&mut self, scene_buffer: &mut Buffer, dag_store: &SceneDAGStore) { 
        let dag = &dag_store.dags[self.dag_key];

        let size = self.entry.get_size();
        let scale = (VOXELS_PER_SHADER_UNIT as u32 / size) as f32; 
        let mat = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale), 
            Quat::IDENTITY,
            Vec3::splat(1.0) - self.entry.offset.as_vec3() / size as f32,
        ).mul_mat4(&self.mat.inverse());

        let inv_mat = mat.inverse();

        let data = SceneDAGObjectData {
            mat: mat.transpose(),
            inv_mat: inv_mat.transpose(),
            
            node_ptr: dag.node_buffer.get_device_address(),
            data_ptr: dag.data_buffer.get_device_address(),
            root_index: self.entry.root_index,
        };

        scene_buffer.copy_data_to_buffer_without_aligment(&[data], self.allocation.start);
    }

    pub fn get_aabb(&self) -> AABB3 {
        let size = self.entry.get_size();
        AABB3::from_min_max(
            &self.mat,
            self.entry.offset.as_vec3a() / VOXELS_PER_SHADER_UNIT as f32, 
            (self.entry.offset.as_vec3a() + Vec3A::splat(size as f32)) / VOXELS_PER_SHADER_UNIT as f32,
        )
    }
}

