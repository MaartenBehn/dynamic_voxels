use std::{mem, time::Instant};

use itertools::Itertools;
use octa_force::log;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelBridge, ParallelIterator};
use smallvec::SmallVec;

use crate::voxel::dag64::{node::VoxelDAG64Node, parallel::{MIN_PAR_LEVEL, ParallelVoxelDAG64}};


impl ParallelVoxelDAG64 {
    pub fn clean(&mut self) {
        mem::swap(&mut self.nodes, &mut self.inactive_nodes);
        mem::swap(&mut self.data, &mut self.inactive_data);

        let mut lock = self.entry_points.lock(); 
        let mut entries = lock.values_mut();

        entries.par_bridge()
            .for_each(|entry|{
                let node = self.clean_recursive(entry.root_index);
                entry.root_index = self.nodes.push(&[node]);
            });

        self.inactive_nodes.reset();
        self.inactive_data.reset();
    } 

    fn clean_recursive_par(&self, index: u32, node_level: u8) -> VoxelDAG64Node {
        let node = self.inactive_nodes.get(index);
        if node.is_leaf() {
            let data = self.inactive_data.get_range(node.range()); 
            return VoxelDAG64Node::new(true, self.data.push(data) as u32, node.pop_mask)
        }

        let new_level = node_level - 1;
        let nodes = node.range()
            .into_par_iter()
            .map(|i| {
                if new_level > MIN_PAR_LEVEL {
                    self.clean_recursive_par(i as u32, new_level)
                } else {
                    self.clean_recursive(i as u32)
                }
            })
            .fold(|| SmallVec::<[_; 64]>::new(), 
                |mut vec, n| {
                    vec.push(n);
                    vec
                })
            .reduce(|| SmallVec::<[_; 64]>::new(), 
                |mut vec_a, vec_b| {
                    vec_a.extend_from_slice(&vec_b);
                    vec_a
                }); 
                
        VoxelDAG64Node::new(false, self.nodes.push(&nodes) as u32, node.pop_mask)
    }

    fn clean_recursive(&self, index: u32) -> VoxelDAG64Node {
        let node = self.inactive_nodes.get(index);
        if node.is_leaf() {
            let data = self.inactive_data.get_range(node.range()); 
            return VoxelDAG64Node::new(true, self.data.push(data) as u32, node.pop_mask)
        }

        let mut nodes = SmallVec::<[_; 64]>::new();
        for i in node.range() {
            nodes.push(self.clean_recursive(i as u32));
        }
        
        VoxelDAG64Node::new(false, self.nodes.push(&nodes) as u32, node.pop_mask)
    }
}
