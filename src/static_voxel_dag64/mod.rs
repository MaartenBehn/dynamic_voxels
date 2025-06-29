use octa_force::{glam::Vec3, log::info};

use crate::{util::to_mb, voxel_grid::VoxelGrid};

pub mod from_voxel_grid;
pub mod renderer;

#[derive(Debug)]
pub struct StaticVoxelDAG64 {
    tree: tree64::Tree64<u8>    
}

impl StaticVoxelDAG64 {
    pub fn get_size(&self) -> Vec3 {
        let size = self.get_size_u32();
        Vec3::splat(size as f32)
    }

    pub fn get_size_u32(&self) -> u32 {
        4_u32.pow(self.tree.root_state().num_levels as u32 - 1)
    }


    pub fn get_root_index(&self) -> u32 {
        self.tree.root_state().index    
    }

    pub fn get_nodes(&self) -> &[tree64::Node] {
        &self.tree.nodes
    }

    pub fn get_nodes_size(&self) -> usize {
        self.tree.nodes.len() * size_of::<tree64::Node>()
    }

    pub fn get_data(&self) -> &[u8] {
        &self.tree.data
    }

    pub fn get_nodes_data_size(&self) -> usize {
        self.tree.data.len() * size_of::<u8>()
    }

    pub fn print_memory_info(&self) { 
        info!("StaticVoxelDAG64: nodes {} MB, data {} MB", 
            to_mb(self.get_nodes_size()),
            to_mb(self.get_nodes_data_size())
        );
    }
}

