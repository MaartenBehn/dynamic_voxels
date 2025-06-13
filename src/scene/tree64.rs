
use octa_force::{glam::{Mat4, Vec3}, vulkan::Buffer, OctaResult};
use slotmap::{new_key_type, SlotMap};

use crate::{buddy_controller::BuddyBufferAllocator, voxel_tree64::VoxelTree64};

new_key_type! { pub struct Tree64Key; }

pub struct Tree64SceneObject {
    pub mat: Mat4,
    pub tree: VoxelTree64,
    pub bvh_index: usize,
    pub alloc_start: usize,
    pub nodes_start: usize, // startof buddy allo
    pub data_start: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tree64SceneObjectData {
    pub data_start: u32,
    pub root_index: u32,
}

impl Tree64SceneObject {
    pub fn new(mat: Mat4, tree: VoxelTree64) -> Self {
        Self {
            mat,
            tree,
            bvh_index: 0,
            alloc_start: 0,
            nodes_start: 0,
            data_start: 0, 
        }
    }

    pub fn push_to_buffer(&mut self, allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<()> {
        let size = size_of::<Tree64SceneObjectData>() + self.tree.tree.nodes.len() + self.tree.tree.data.len(); 
        let (start, _) = allocator.alloc(size)?;
        self.alloc_start = start;

        self.nodes_start = self.alloc_start + size_of::<Tree64SceneObjectData>();
        self.data_start = self.nodes_start + self.tree.tree.nodes.len(); 

        let data = Tree64SceneObjectData {
            data_start: self.data_start as _,
            root_index: self.tree.tree.root_state().index,
        };

        buffer.copy_data_to_buffer_without_aligment(&[data], self.alloc_start)?;
        buffer.copy_data_to_buffer_without_aligment(&self.tree.tree.nodes, self.nodes_start)?;
        buffer.copy_data_to_buffer_without_aligment(&self.tree.tree.data, self.data_start)?;

        Ok(())
    }
}

