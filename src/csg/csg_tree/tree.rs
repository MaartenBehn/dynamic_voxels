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

pub struct UnionResult {
    pub union_node_index: CSGTreeIndex,
    pub new_object_index: CSGTreeIndex,
}

pub struct CutResult {
    pub cut_node_index: CSGTreeIndex,
    pub base_index: CSGTreeIndex,
    pub new_object_index: CSGTreeIndex,
}

impl<V: Base, C: MC<D>, const D: usize> CSGTree<V, C, D> { 
    pub fn from_node(node: CSGTreeNode<V, C, D>) -> Self {
        Self {
            nodes: vec![node],
            changed: true,
            root: 0,
        }
    }

    pub fn union_node_at_root(&mut self, node: CSGTreeNode<V, C, D>) -> UnionResult {   
        let new_index = self.nodes.len();
        self.nodes.push(node);
        self.changed = true;

        if self.nodes.len() == 1 {
            self.root = new_index;
            return UnionResult {
                union_node_index: new_index,
                new_object_index: new_index,
            };
        }

        let root_node = &mut self.nodes[self.root];
        if let CSGTreeNodeData::Union(union) = &mut root_node.data {
            union.add_node(new_index);
            self.nodes[new_index].parent = self.root;
            
            return UnionResult {
                union_node_index: self.root,
                new_object_index: new_index,
            };
        }

        let union_index = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_union(vec![self.root, new_index]));
        
        self.nodes[new_index].parent = union_index;
        self.nodes[self.root].parent = union_index;

        self.root = union_index;
        
        UnionResult { 
            union_node_index: union_index, 
            new_object_index: new_index 
        }
    }

    pub fn union_node_at_index(&mut self, node: CSGTreeNode<V, C, D>, index: CSGTreeIndex) -> UnionResult {
        let new_index = self.nodes.len();
        self.nodes.push(node);
        self.changed = true;

        let current_node = &mut self.nodes[index];
        if let CSGTreeNodeData::Union(union) = &mut current_node.data {
            union.add_node(new_index);
            self.nodes[new_index].parent = index;

            return UnionResult {
                union_node_index: index,
                new_object_index: new_index,
            };
        }
        
        let parent = current_node.parent;
        let union_index = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_union(vec![index, new_index]));

        self.update_child(parent, index, union_index);
        self.nodes[union_index].parent = parent;

        self.nodes[new_index].parent = union_index;
        self.nodes[index].parent = union_index;

        UnionResult {
            union_node_index: union_index,
            new_object_index: new_index,
        }
    }

    pub fn cut_node_at_root(&mut self, node: CSGTreeNode<V, C, D>) -> CutResult {
        let root_node = &self.nodes[self.root];
        if let CSGTreeNodeData::Remove(cut) = &root_node.data {
            let remove_index = cut.remove;
            let base_index = cut.base;
            let res = self.union_node_at_index(node, remove_index);
            self.nodes[res.union_node_index].parent = self.root;

            return CutResult {
                cut_node_index: self.root,
                base_index,
                new_object_index: res.new_object_index,
            };
        }

        assert!(!self.nodes.is_empty(), "You can not remove from an empty CSGTree");

        let new_index = self.nodes.len();
        self.nodes.push(node);

        let cut_index = self.nodes.len();
        let base_index = self.root;
        self.nodes.push(CSGTreeNode::new_remove(base_index, new_index));
        
        self.nodes[new_index].parent = cut_index;
        self.nodes[base_index].parent = cut_index;

        self.root = cut_index;

        CutResult {
            cut_node_index: cut_index,
            base_index,
            new_object_index: new_index,
        }
    }

    pub fn cut_node_at_index(&mut self, node: CSGTreeNode<V, C, D>, index: CSGTreeIndex) -> CutResult {
        let current_node = &self.nodes[index];
        if let CSGTreeNodeData::Remove(cut) = &current_node.data {
            let remove_index = cut.remove;
            let base_index = cut.base;
            let res = self.union_node_at_index(node, remove_index);
            self.nodes[res.union_node_index].parent = index;

            return CutResult {
                cut_node_index: index,
                base_index,
                new_object_index: res.new_object_index,
            };
        }

        assert!(!self.nodes.is_empty(), "You can not remove from an empty CSGTree");

        let parent = current_node.parent;

        let new_index = self.nodes.len();
        self.nodes.push(node);

        let cut_index = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_remove(index, new_index));

        self.update_child(parent, index, cut_index);
        self.nodes[cut_index].parent = parent;
        
        self.nodes[new_index].parent = cut_index;
        self.nodes[index].parent = cut_index;
        
        CutResult {
            cut_node_index: cut_index,
            base_index: index,
            new_object_index: new_index,
        }
    }

    fn update_child(&mut self, index: CSGTreeIndex, old: CSGTreeIndex, new: CSGTreeIndex) {
        let node = &mut self.nodes[index];

        match &mut node.data {
            CSGTreeNodeData::Union(union) => {
                let i = union.indecies.iter()
                    .position(|index| *index == old)
                    .expect("Union Parent had no child");
                union.indecies[i] = new;
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
