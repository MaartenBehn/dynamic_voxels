use bvh::{bvh::Bvh, flat_bvh::FlatNode};
use octa_force::glam::Mat4;

use crate::{csg::{r#box::CSGBox, sphere::CSGSphere, Base}, util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{remove::CSGTreeRemove, union::CSGTreeUnion};

pub type CSGTreeIndex = usize; 
pub const CSG_TREE_INDEX_INVALID: CSGTreeIndex = CSGTreeIndex::MAX;

#[derive(Debug, Clone)]
pub enum CSGTreeNodeData<M, V: Ve<T, D>, T: Nu, const D: usize> {
    Union(CSGTreeUnion<V, T, D>),
    Remove(CSGTreeRemove),
   
    None,
    Box(CSGBox<M, V, T, D>),
    Sphere(CSGSphere<M, V, T, D>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

#[derive(Debug, Clone)]
pub struct CSGTreeNode<M, V: Ve<T, D>, T: Nu, const D: usize> {
    pub data: CSGTreeNodeData<M, V, T, D>,
    pub parent: CSGTreeIndex,
}

#[derive(Debug, Clone, Default)]
pub struct CSGTree<M, V: Ve<T, D>, T: Nu, const D: usize> {
    pub nodes: Vec<CSGTreeNode<M, V, T, D>>,
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

impl<M: Base, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {  
    pub fn from_node(node: CSGTreeNode<M, V, T, D>) -> Self {
        Self {
            nodes: vec![node],
            changed: true,
            root: 0,
        }
    }

    pub fn union_at_root(&mut self, other: &[CSGTreeNode<M, V, T, D>], other_root: usize) -> UnionResult {   
        assert!(!other.is_empty());
        self.changed = true;


        if self.nodes.is_empty() {
            self.nodes.extend_from_slice(other);
            self.root = other_root;

            return UnionResult {
                union_node_index: other_root,
                new_object_index: other_root,
            };
        }

        if let CSGTreeNodeData::None = &self.nodes[self.root].data {
            self.nodes = Vec::with_capacity(other.len());
            self.nodes.extend_from_slice(other);
            self.root = other_root;

            return UnionResult {
                union_node_index: other_root,
                new_object_index: other_root,
            };
        }

        let length = self.nodes.len();
        let new_index = other_root + length;
        self.nodes.extend_from_slice(other);
        shift_node_indecies(&mut self.nodes[length..], length);

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

    pub fn union_at_index(&mut self, index: CSGTreeIndex, other: &[CSGTreeNode<M, V, T, D>], other_root: usize) -> UnionResult {
        self.changed = true;

        let length = self.nodes.len();
        let new_index = other_root + length;
        self.nodes.extend_from_slice(other);
        shift_node_indecies(&mut self.nodes[length..], length);

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

    pub fn cut_at_root(&mut self, other: &[CSGTreeNode<M, V, T, D>], other_root: usize) -> CutResult { 
        if other.is_empty() {
            return CutResult {
                cut_node_index: self.root,
                base_index: self.root,
                new_object_index: self.root,
            };
        }

        self.changed = true;

        if self.nodes.is_empty() {
            self.nodes.push(CSGTreeNode::new(CSGTreeNodeData::None, CSG_TREE_INDEX_INVALID));
        }

        let root_node = &self.nodes[self.root];
        if let CSGTreeNodeData::Remove(cut) = &root_node.data {
            let remove_index = cut.remove;
            let base_index = cut.base;
            let res = self.union_at_index(remove_index, other, other_root);
            self.nodes[res.union_node_index].parent = self.root;

            return CutResult {
                cut_node_index: self.root,
                base_index,
                new_object_index: res.new_object_index,
            };
        }

        assert!(!self.nodes.is_empty(), "You can not remove from an empty CSGTree");

        let length = self.nodes.len();
        let new_index = other_root + length;
        self.nodes.extend_from_slice(other);
        shift_node_indecies(&mut self.nodes[length..], length);

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

    pub fn cut_at_index(&mut self, index: CSGTreeIndex, other: &[CSGTreeNode<M, V, T, D>], other_root: usize) -> CutResult {
        self.changed = true;

        let current_node = &self.nodes[index];
        if let CSGTreeNodeData::Remove(cut) = &current_node.data {
            let remove_index = cut.remove;
            let base_index = cut.base;
            let res = self.union_at_index(remove_index, other, other_root);
            self.nodes[res.union_node_index].parent = index;

            return CutResult {
                cut_node_index: index,
                base_index,
                new_object_index: res.new_object_index,
            };
        }

        assert!(!self.nodes.is_empty(), "You can not remove from an empty CSGTree");

        let parent = current_node.parent;

        let length = self.nodes.len();
        let new_index = other_root + length;
        self.nodes.extend_from_slice(other);
        shift_node_indecies(&mut self.nodes[length..], length);

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

// Can be used to offset the store indecies of nodes so they align when appended.
pub fn shift_node_indecies<M: Base, V: Ve<T, D>, T: Nu, const D: usize>(nodes: &mut [CSGTreeNode<M, V, T, D>], ammount: usize) {
    for node in nodes {
        match &mut node.data {
            CSGTreeNodeData::Union(csgtree_union) => csgtree_union.shift_indecies(ammount),
            CSGTreeNodeData::Remove(csgtree_remove) => csgtree_remove.shift_indecies(ammount),
            _ => {}
        }

        if node.parent != CSG_TREE_INDEX_INVALID {
            node.parent += ammount;
        } 
    }
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> CSGTreeNode<M, V, T, D> {
    pub fn new(data: CSGTreeNodeData<M, V, T, D>, parent: CSGTreeIndex) -> Self {
        Self {
            data,
            parent,
        }
    }
}
