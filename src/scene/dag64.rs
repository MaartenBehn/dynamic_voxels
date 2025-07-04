
use octa_force::{glam::{vec3, vec4, Mat4, Quat, Vec3, Vec4, Vec4Swizzles}, log::debug, vulkan::Buffer, OctaResult};
use slotmap::{new_key_type, SlotMap};

use crate::{multi_data_buffer::buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, voxel::dag64::VoxelDAG64};

new_key_type! { pub struct Tree64Key; }

#[derive(Debug)]
pub struct DAG64SceneObject {
    pub mat: Mat4,
    pub dag: VoxelDAG64,
    pub bvh_index: usize,
    pub allocation: Option<BuddyAllocation>,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DAG64SceneObjectData {
    pub x_axis: Vec3,
    pub root_index: u32,
    pub y_axis: Vec3,
    pub node_start: u32,
    pub z_axis: Vec3,
    pub data_start: u32,
    pub w_axis: Vec3,
    pub fill: u32,
    pub inv_mat: Mat4
}

impl DAG64SceneObject {
    pub fn new(mat: Mat4, dag: VoxelDAG64) -> Self {
        Self {
            mat,
            dag,
            bvh_index: 0,
            allocation: None,
        }
    }

    pub fn push_to_buffer(&mut self, allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<()> {
        self.allocation = Some(allocator.alloc(size_of::<DAG64SceneObjectData>())?);

        let nodes_start = self.get_allocation().start + size_of::<DAG64SceneObjectData>();

        let mat = Mat4::from_scale_rotation_translation(
            Vec3::ONE / self.dag.get_size(), 
            Quat::IDENTITY,
            Vec3::splat(1.5),
        ).mul_mat4(&self.mat.inverse());

        let data = DAG64SceneObjectData {
            x_axis: mat.x_axis.xyz(),
            y_axis: mat.y_axis.xyz(),
            z_axis: mat.z_axis.xyz(),
            w_axis: mat.w_axis.xyz(),
            inv_mat: mat.inverse().transpose(),
            
            root_index: self.dag.root_index as _,
            node_start: 0,
            data_start: 0,
            
            fill: 0,
        };

        //dbg!(&self.dag);

        buffer.copy_data_to_buffer_without_aligment(&[data], self.get_allocation().start);
        self.dag.nodes.flush(buffer);
        self.dag.data.flush(buffer);

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
