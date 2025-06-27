
use octa_force::{glam::{vec3, vec4, Mat4, Quat, Vec3, Vec4, Vec4Swizzles}, log::debug, vulkan::Buffer, OctaResult};
use slotmap::{new_key_type, SlotMap};

use crate::{multi_data_buffer::buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, scene::dag64::DAG64SceneObjectData, static_voxel_dag64::StaticVoxelDAG64};

new_key_type! { pub struct Tree64Key; }

#[derive(Debug)]
pub struct StaticDAG64SceneObject {
    pub mat: Mat4,
    pub dag: StaticVoxelDAG64,
    pub bvh_index: usize,
    pub allocation: Option<BuddyAllocation>,
}

impl StaticDAG64SceneObject {
    pub fn new(mat: Mat4, tree: StaticVoxelDAG64) -> Self {
        Self {
            mat,
            dag: tree,
            bvh_index: 0,
            allocation: None,
        }
    }

    pub fn push_to_buffer(&mut self, allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<()> {
        let size = size_of::<DAG64SceneObjectData>() + self.dag.get_nodes_size() + self.dag.get_nodes_data_size();
        debug!("Tree64 Obvject Size: {size}");

        self.allocation = Some(allocator.alloc(size)?);

        let nodes_start = self.get_allocation().start + size_of::<DAG64SceneObjectData>();
        let data_start = self.get_allocation().start + size_of::<DAG64SceneObjectData>() + self.dag.get_nodes_size(); 

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
            
            root_index: self.dag.get_root_index() as _,
            node_start: nodes_start as _,
            data_start: data_start as _,
            
            fill: 0,
        };

        buffer.copy_data_to_buffer_without_aligment(&[data], self.get_allocation().start);
        buffer.copy_data_to_buffer_without_aligment(&self.dag.get_nodes(), nodes_start);
        buffer.copy_data_to_buffer_without_aligment(&self.dag.get_data(), data_start);

        Ok(())
    }
}

impl StaticDAG64SceneObject {
    pub fn get_mut_allocation(&mut self) -> &mut BuddyAllocation {
        self.allocation.as_mut().unwrap()
    }

    pub fn get_allocation(&self) -> &BuddyAllocation {
        self.allocation.as_ref().unwrap()
    }
}
