
use std::{rc::Rc, sync::Arc};

use octa_force::{glam::{vec3, vec4, Mat4, Quat, Vec3, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles}, log::debug, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{multi_data_buffer::buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, util::aabb::AABB, voxel::dag64::{DAG64EntryKey, VoxelDAG64}, VOXELS_PER_METER, VOXELS_PER_SHADER_UNIT};

use super::{Scene, SceneObjectData, SceneObjectKey, SceneObjectType};

new_key_type! { pub struct Tree64Key; }

#[derive(Debug)]
pub struct DAG64SceneObject {
    pub mat: Mat4,
    pub dag: Arc<Mutex<VoxelDAG64>>,
    pub entry_key: DAG64EntryKey,
    pub allocation: BuddyAllocation,
    pub node_buffer: Buffer,
    pub data_buffer: Buffer,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DAG64SceneObjectData {
    pub mat: Mat4,
    pub inv_mat: Mat4,
    pub node_ptr: u64,
    pub data_ptr: u64,
    pub root_index: u32,
}

impl Scene {
    pub fn add_dag64(
        &mut self, 
        context: &Context, 
        mat: Mat4, 
        entry_key: DAG64EntryKey, 
        dag: Arc<Mutex<VoxelDAG64>>
    ) -> OctaResult<SceneObjectKey> {
        
        let object = SceneObjectType::DAG64(DAG64SceneObject::new(context, mat, entry_key, dag, &mut self.allocator)?); 
        Ok(self.add_object(object))
    }
}

impl DAG64SceneObject {
    pub fn new(context: &Context, mat: Mat4, entry_key: DAG64EntryKey, dag: Arc<Mutex<VoxelDAG64>>, allocator: &mut BuddyBufferAllocator) -> OctaResult<Self> {
        
        let dag_ref = dag.lock();

        let mut node_buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            MemoryLocation::CpuToGpu,
            dag_ref.nodes.get_memory_size() as _,
        )?;

        let mut data_buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            MemoryLocation::CpuToGpu,
            dag_ref.data.get_memory_size() as _,
        )?;
        drop(dag_ref);

        let allocation = allocator.alloc(size_of::<DAG64SceneObjectData>())?;

        Ok(Self {
            mat,
            dag,
            allocation,
            entry_key,
            node_buffer,
            data_buffer,
        })
    }

    pub fn push_to_buffer(&mut self, scene_buffer: &mut Buffer) { 
        
        let mut dag = self.dag.lock(); 
        let entry = dag.entry_points[self.entry_key];

        let size = entry.get_size();
        let scale = (VOXELS_PER_SHADER_UNIT as u32 / size) as f32; 
        let mat = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale), 
            Quat::IDENTITY,
            Vec3::splat(1.0) - Vec3::from(entry.offset) / size as f32,
        ).mul_mat4(&self.mat.inverse());

        let inv_mat = mat.inverse();

        let data = DAG64SceneObjectData {
            mat: mat.transpose(),
            inv_mat: inv_mat.transpose(),
            
            node_ptr: self.node_buffer.get_device_address(),
            data_ptr: self.data_buffer.get_device_address(),
            root_index: entry.root_index,
        };

        scene_buffer.copy_data_to_buffer_without_aligment(&[data], self.allocation.start);

        dag.nodes.flush(&mut self.node_buffer);
        dag.data.flush(&mut self.data_buffer);
    }

    pub fn get_aabb(&self) -> AABB {
        let dag = self.dag.lock(); 
        let entry = dag.entry_points[self.entry_key];
        let size = entry.get_size();
        AABB::from_min_max(
            &self.mat,
            entry.offset / VOXELS_PER_SHADER_UNIT as f32, 
            (entry.offset + Vec3A::splat(size as f32)) / VOXELS_PER_SHADER_UNIT as f32,
        )
    }
}

