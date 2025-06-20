
use octa_force::{glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles}, log::debug, vulkan::Buffer, OctaResult};
use slotmap::{new_key_type, SlotMap};

use crate::{buddy_controller::BuddyBufferAllocator, voxel_tree64::VoxelTree64};

new_key_type! { pub struct Tree64Key; }

#[derive(Debug)]
pub struct Tree64SceneObject {
    pub mat: Mat4,
    pub tree: VoxelTree64,
    pub bvh_index: usize,
    pub alloc_start: usize,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Tree64SceneObjectData {
    pub x_axis: Vec3,
    pub data_offset: u32,
    pub y_axis: Vec3,
    pub root_index: u32,
    pub z_axis: Vec3,
    fill_1: u32,
    pub w_axis: Vec3,
    fill_2: u32,
    pub inv_mat: Mat4
}

impl Tree64SceneObject {
    pub fn new(mat: Mat4, tree: VoxelTree64) -> Self {
        Self {
            mat,
            tree,
            bvh_index: 0,
            alloc_start: 0,
        }
    }

    pub fn push_to_buffer(&mut self, allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<()> {
        let size = size_of::<Tree64SceneObjectData>() + self.tree.get_nodes_size() + self.tree.get_nodes_data_size();
        debug!("Tree64 Obvject Size: {size}");

        let (start, _) = allocator.alloc(size)?;
        self.alloc_start = start;

        let nodes_start = self.alloc_start + size_of::<Tree64SceneObjectData>();
        let data_offset = size_of::<Tree64SceneObjectData>() + self.tree.get_nodes_size(); 
        let data_start = self.alloc_start + data_offset; 

        let mat = Mat4::from_scale_rotation_translation(
            Vec3::ONE / self.tree.get_size(), 
            Quat::IDENTITY,
            //Quat::from_euler(octa_force::glam::EulerRot::XYZ, 0.0, 0.0, f32::to_radians(45.0)),
            Vec3::splat(1.5),
        ).mul_mat4(&self.mat.inverse());

        let data = Tree64SceneObjectData {
            x_axis: mat.x_axis.xyz(),
            y_axis: mat.y_axis.xyz(),
            z_axis: mat.z_axis.xyz(),
            w_axis: mat.w_axis.xyz(),
            data_offset: data_offset as _,
            root_index: self.tree.get_root_index() as _,
            fill_1: 0,
            fill_2: 0,
            inv_mat: mat.inverse(),
        };

        buffer.copy_data_to_buffer_without_aligment(&[data], self.alloc_start)?;
        buffer.copy_data_to_buffer_without_aligment(&self.tree.get_nodes(), nodes_start)?;
        buffer.copy_data_to_buffer_without_aligment(&self.tree.get_data(), data_start)?;

        Ok(())
    }
}

