use core::fmt;

use octa_force::{anyhow::{anyhow, bail, ensure, Context}, OctaResult};
use slotmap::Key;

use super::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};


impl<T: fmt::Debug> SlotMapCSGTree<T> {
    pub fn append_node_with_union(&mut self, node: SlotMapCSGNode<T>) -> SlotMapCSGTreeKey {
        let new_index = self.nodes.insert(node);
        self.root_node = self.nodes.insert(SlotMapCSGNode::new(
            SlotMapCSGNodeData::Union(self.root_node, new_index) 
        ));

        new_index
    }

    pub fn remove_node_as_child_of_union(&mut self, index: SlotMapCSGTreeKey) -> OctaResult<()> {
        let node = self.nodes
            .remove(index)
            .ok_or(anyhow!("Cant remove Node from Tree because the index is not valid!"))?;

        let union_node = self.nodes
            .remove(node.parent)
            .ok_or(anyhow!("Parent Index of to be removed not was not valid!"))?;

        let other_index = if let SlotMapCSGNodeData::Union(c1, c2) = union_node.data {
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

        if union_node.parent == SlotMapCSGTreeKey::null() {
            self.root_node = other_index;
        } else {
            self.replace_index_in_node(union_node.parent, node.parent, other_index)
                .context("Failed to replace the index of the union node in the parent of the union node!")?;
        }

        Ok(())
    }

    fn replace_index_in_node(&mut self, index: SlotMapCSGTreeKey, search: SlotMapCSGTreeKey, replace: SlotMapCSGTreeKey) -> OctaResult<()> {
        let node = self.nodes
            .get_mut(index)
            .ok_or(anyhow!("Node in which a index should be replaced is not there!"))?;

        match &mut node.data {
            SlotMapCSGNodeData::Union(c1, c2)
            | SlotMapCSGNodeData::Remove(c1, c2)
            | SlotMapCSGNodeData::Intersect(c1, c2) => {
                if *c1 == search {
                    *c1 = replace;    
                } else if *c2 == search {
                    *c2 = replace;
                } else {
                    bail!("The search index did not match any child index in the node {:?}!", node.data);
                }
            },
            SlotMapCSGNodeData::Mat(_, c) => {
                ensure!(*c == search, "The child index in the mat Node did not match the search index!");

                *c = replace;
            },
            SlotMapCSGNodeData::Box(_, _)
            | SlotMapCSGNodeData::Sphere(_, _)
            | SlotMapCSGNodeData::All(_) => todo!(),
            | SlotMapCSGNodeData::VoxelGrid(_, _) => {
                bail!("Can not replace a child in index in a node {:?}, because it has no children!", node.data)
            }
        }

        Ok(())
    }

}
