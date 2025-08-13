use core::fmt;

use crate::volume::remove_trait::VolumeRemove;

use super::tree::{CSGNode2D, CSGNodeData2D, CSGTree2D, CSGTreeKey2D};


impl<T: fmt::Debug> VolumeRemove for CSGTree2D<T> {
    fn remove_volume(&mut self, other: Self) {
        if other.nodes.len() == 1 {
            let node = other.nodes.into_iter().next().unwrap().1;
            self.append_node_with_remove(node);
        } else {
            todo!()
        }
    }
}

impl<T: fmt::Debug> CSGTree2D<T> { 
    pub fn append_node_with_remove(&mut self, node: CSGNode2D<T>) -> CSGTreeKey2D {
        let new_index = self.nodes.insert(node);
        self.root_node = self.nodes.insert(CSGNode2D::new(
            CSGNodeData2D::Remove(self.root_node, new_index) 
        ));

        new_index
    }
}

