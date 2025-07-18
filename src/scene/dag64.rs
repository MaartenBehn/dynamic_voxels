
use octa_force::{glam::{vec3, vec4, Mat4, Quat, Vec3, Vec4, Vec4Swizzles}, log::debug, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use slotmap::{new_key_type, SlotMap};

use crate::{multi_data_buffer::buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, voxel::dag64::{DAG64EntryKey, VoxelDAG64}, VOXELS_PER_SHADER_UNIT};

new_key_type! { pub struct Tree64Key; }

#[derive(Debug)]
pub struct DAG64SceneObject {
    pub mat: Mat4,
    pub dag: VoxelDAG64,
    pub entry_key: DAG64EntryKey,
    pub bvh_index: usize,
    pub allocation: Option<BuddyAllocation>,
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

impl DAG64SceneObject {
    pub fn new(context: &Context, mat: Mat4, entry_key: DAG64EntryKey, dag: VoxelDAG64) -> OctaResult<Self> {
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

        Ok(Self {
            mat,
            dag,
            bvh_index: 0,
            allocation: None,
            entry_key,
            node_buffer,
            data_buffer,
        })
    }

    pub fn push_to_buffer(&mut self, allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<()> {
        self.allocation = Some(allocator.alloc(size_of::<DAG64SceneObjectData>())?);

            
        let entry = &self.dag.entry_points[self.entry_key];

        let mat = Mat4::from_scale_rotation_translation(
            Vec3::splat((VOXELS_PER_SHADER_UNIT as u32 / entry.get_size_u32()) as f32 ), 
            Quat::IDENTITY,
            Vec3::splat(1.5),
        ).mul_mat4(&self.mat.inverse());
        let inv_mat = mat.inverse();

        let data = DAG64SceneObjectData {
            mat: mat.transpose(),
            inv_mat: inv_mat.transpose(),
            
            node_ptr: self.node_buffer.get_device_address(),
            data_ptr: self.data_buffer.get_device_address(),
            root_index: entry.root_index,
        };

        //dbg!(&self.dag);

        buffer.copy_data_to_buffer_without_aligment(&[data], self.get_allocation().start);

        self.dag.nodes.flush(&mut self.node_buffer);
        self.dag.data.flush(&mut self.data_buffer);

        self.dag.print_memory_info();

        Ok(())
    }
}

impl DAG64SceneObject {
    pub fn get_mut_allocation(&mut self) -> &mut BuddyAllocation {
        self.allocation.as_mut().unwrap()
    }

    pub fn get_allocation(&self) -> &BuddyAllocation {
        self.allocation.as_ref().unwrap()
    }
}
