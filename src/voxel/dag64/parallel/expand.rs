use octa_force::{glam::UVec3, OctaResult};

use crate::{util::iaabb3d::AABBI, voxel::dag64::{node::VoxelDAG64Node, DAG64EntryData, DAG64EntryKey}};
use super::ParallelVoxelDAG64;

impl ParallelVoxelDAG64 {
    pub fn expand_to_include_aabb(&mut self, based_on_entry: DAG64EntryKey, aabb: AABBI) -> OctaResult<DAG64EntryData> {
        let mut entry_data = self.entry_points.lock()[based_on_entry].to_owned(); 

        let mut size = 4_i32.pow(entry_data.levels as u32);
        let mut tree_aabb = AABBI::new(entry_data.offset, entry_data.offset + size);

        let model_center = aabb.center();

        // Increase the Tree if the model does not fit.
        // MAYBE If the model_aabb is not to big for the tree_aabb but just sticking out.
        // It would be possible to move the tree_aabb. 
        // But this would mean the entire tree would need to be regenerated. 
        while !tree_aabb.contains_aabb(aabb) {

            // The + 2 says that the 3rd cell is the center so the old tree will placed in the
            // middle of the new level.
            let child_pos = ((model_center - tree_aabb.min) / size) + 2;
            let child_index = child_pos.as_uvec3().dot(UVec3::new(1, 4, 16));

            let new_root = VoxelDAG64Node::new(false, entry_data.root_index, 1 << child_index as u64);
            entry_data.root_index = self.nodes.push(&[new_root])?;
            
            entry_data.offset = entry_data.offset - child_pos * size; 
            entry_data.levels += 1;
            size = 4_i32.pow(entry_data.levels as u32);
            tree_aabb = AABBI::new(entry_data.offset, entry_data.offset + size);
        }
        
        Ok(entry_data)
    }
}
