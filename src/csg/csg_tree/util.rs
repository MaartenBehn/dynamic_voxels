use core::fmt;

use octa_force::{anyhow::{anyhow, bail, ensure, Context}, OctaResult};
use slotmap::Key;

use super::tree::{CSGNode, CSGNodeData, CSGTree, CSGTreeKey};


impl<T: fmt::Debug> CSGTree<T> {
    pub fn append_node_with_union(&mut self, node: CSGNode<T>) -> CSGTreeKey {
        let new_index = self.nodes.insert(node);
        self.root_node = self.nodes.insert(CSGNode::new(
            CSGNodeData::Union(self.root_node, new_index) 
        ));

        new_index
    }

    pub fn append_node_with_remove(&mut self, node: CSGNode<T>) -> CSGTreeKey {
        let new_index = self.nodes.insert(node);
        self.root_node = self.nodes.insert(CSGNode::new(
            CSGNodeData::Remove(self.root_node, new_index) 
        ));

        new_index
    }

    pub fn remove_node_as_child_of_union(&mut self, index: CSGTreeKey) -> OctaResult<()> {
        let node = self.nodes
            .remove(index)
            .ok_or(anyhow!("Cant remove Node from Tree because the index is not valid!"))?;

        let union_node = self.nodes
            .remove(node.parent)
            .ok_or(anyhow!("Parent Index of to be removed not was not valid!"))?;

        let other_index = if let CSGNodeData::Union(c1, c2) = union_node.data {
            if c1 == index { 
                c2 
            } else if c2 == index { 
                c1 
            } else { 
                bail!("None of the children of the Union that was marked as parent contained the index of the node!"); 
            }
        } else {
            bail!("The parent of the node was not a union!");
        };

        if union_node.parent == CSGTreeKey::null() {
            self.root_node = other_index;
        } else {
            self.replace_index_in_node(union_node.parent, node.parent, other_index)
                .context("Failed to replace the index of the union node in the parent of the union node!")?;
        }

        Ok(())
    }

    fn replace_index_in_node(&mut self, index: CSGTreeKey, search: CSGTreeKey, replace: CSGTreeKey) -> OctaResult<()> {
        let node = self.nodes
            .get_mut(index)
            .ok_or(anyhow!("Node in which a index should be replaced is not there!"))?;

        match &mut node.data {
            CSGNodeData::Union(c1, c2)
            | CSGNodeData::Remove(c1, c2)
            | CSGNodeData::Intersect(c1, c2) => {
                if *c1 == search {
                    *c1 = replace;    
                } else if *c2 == search {
                    *c2 = replace;
                } else {
                    bail!("The search index did not match any child index in the node {:?}!", node.data);
                }
            },
            CSGNodeData::Box(..)
            | CSGNodeData::Sphere(..)
            | CSGNodeData::All(..)
            | CSGNodeData::OffsetVoxelGrid(..)
            | CSGNodeData::SharedVoxelGrid(..)=> {
                bail!("Can not replace a child in index in a node {:?}, because it has no children!", node.data)
            }
        }

        Ok(())
    }
}

impl<T> CSGTree<T> {
    pub fn get_id_parents(&self, ids: &[CSGTreeKey]) -> Vec<CSGTreeKey> {
        self.nodes
            .iter()
            .filter_map(|(i, node)| {
                match node.data {
                    CSGNodeData::Union(child1, child2)
                    | CSGNodeData::Remove(child1, child2)
                    | CSGNodeData::Intersect(child1, child2) => {
                        if ids.contains(&child1) || ids.contains(&child2) {
                            return Some(i);
                        }
                    }
                    _ => {}
                }

                None
            })
            .collect()
    }

    pub fn get_value_index_of_key(&self, key: CSGTreeKey) -> usize {
        self.nodes.keys()
            .position(|k| k == key)
            .unwrap()
    }
}
