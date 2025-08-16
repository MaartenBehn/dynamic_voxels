use bvh::{bvh::Bvh, flat_bvh::FlatNode};
use octa_force::glam::Mat4;

use crate::{csg::{r#box::CSGBox, sphere::CSGSphere, Base}, util::{aabb::AABB, math_config::MC}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{remove::CSGTreeRemove, union::CSGTreeUnion};

pub type CSGTreeIndex = usize; 
pub const CSG_TREE_INDEX_INVALID: CSGTreeIndex = CSGTreeIndex::MAX;

#[derive(Debug, Clone)]
pub enum CSGTreeNodeData<V, C: MC<D>, const D: usize> {
    Union(CSGTreeUnion<C, D>),
    Remove(CSGTreeRemove),
    
    Box(CSGBox<V, C, D>),
    Sphere(CSGSphere<V, C, D>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

#[derive(Debug, Clone)]
pub struct CSGTreeNode<V, C: MC<D>, const D: usize> {
    pub data: CSGTreeNodeData<V, C, D>,
    pub parent: CSGTreeIndex,
}

#[derive(Debug, Clone, Default)]
pub struct CSGTree<V, C: MC<D>, const D: usize> {
    pub nodes: Vec<CSGTreeNode<V, C, D>>,
    pub changed: bool,
    pub root: CSGTreeIndex,
}

impl<V: Base, C: MC<D>, const D: usize> CSGTree<V, C, D> { 
    pub fn from_node(node: CSGTreeNode<V, C, D>) -> Self {
        Self {
            nodes: vec![node],
            changed: true,
            root: 0,
        }
    }

    pub fn add_node_at_root(&mut self, node: CSGTreeNode<V, C, D>) -> CSGTreeIndex {   
        let new_index = self.nodes.len();
        self.nodes.push(node);
        self.changed = true;

        if self.nodes.len() == 1 {
            self.root = new_index;
            return new_index;
        }

        let root_node = &mut self.nodes[self.root];
        if let CSGTreeNodeData::Union(union) = &mut root_node.data {
            union.add_node(new_index);
            return new_index;
        }

        let union_index = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_union(vec![self.root, new_index]));
        self.root = union_index;
        new_index
    }

    pub fn add_node_at_index(&mut self, node: CSGTreeNode<V, C, D>, index: CSGTreeIndex) -> CSGTreeIndex {
        let new_index = self.nodes.len();
        self.nodes.push(node);
        self.changed = true;

        let current_node = &mut self.nodes[index];
        if let CSGTreeNodeData::Union(union) = &mut current_node.data {
            union.add_node(index);
            return new_index;
        }
        
        let parent = current_node.parent;
        let union_index = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_union(vec![index, new_index]));

        self.update_child(parent, index, union_index);

        new_index
    }

    pub fn remove_node_at_root(&mut self, node: CSGTreeNode<V, C, D>) -> CSGTreeIndex {
        let root_node = &self.nodes[self.root];
        if let CSGTreeNodeData::Remove(remove) = &root_node.data {
            let remove_index = remove.remove;
            return self.add_node_at_index(node, remove_index);
        }

        assert!(!self.nodes.is_empty(), "You can not remove from an empty CSGTree");

        let new_index = self.nodes.len();
        self.nodes.push(node);

        let remove_index = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_remove(self.root, new_index));
        self.root = remove_index;
        new_index
    }

    pub fn remove_node_at_index(&mut self, node: CSGTreeNode<V, C, D>, index: CSGTreeIndex) -> CSGTreeIndex {
        let current_node = &self.nodes[index];
        if let CSGTreeNodeData::Remove(remove) = &current_node.data {
            let remove_index = remove.remove;
            return self.add_node_at_index(node, remove_index);
        }

        assert!(!self.nodes.is_empty(), "You can not remove from an empty CSGTree");

        let parent = current_node.parent;

        let new_index = self.nodes.len();
        self.nodes.push(node);

        let remove_index = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_remove(index, new_index));

        self.update_child(parent, index, remove_index);
        
        new_index
    }

    fn update_child(&mut self, index: CSGTreeIndex, old: CSGTreeIndex, new: CSGTreeIndex) {
        let node = &mut self.nodes[index];

        match &mut node.data {
            CSGTreeNodeData::Union(union) => {
                let node = union.nodes.iter_mut()
                    .find(|node| node.index == old)
                    .expect("Union Parent had no child");
                node.index = new;
            },
            CSGTreeNodeData::Remove(remove) => {
                if remove.base == old {
                    remove.base = new;
                } else if remove.remove == old {
                    remove.remove = new;
                } else {
                    panic!("Remove Parent had no child");
                }
            },
            _ => unreachable!()
        }
    }
}

impl<V, C: MC<D>, const D: usize> CSGTreeNode<V, C, D> {
    pub fn new(data: CSGTreeNodeData<V, C, D>, parent: CSGTreeIndex) -> Self {
        Self {
            data,
            parent,
        }
    }
}
