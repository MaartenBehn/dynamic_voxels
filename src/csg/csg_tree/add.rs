use crate::{csg::{Base, csg_tree::tree::CSG_TREE_INDEX_INVALID}, util::{number::Nu, vector::Ve}, voxel::grid::shared::SharedVoxelGrid};

use super::tree::{CSGTree, CSGTreeIndex, CSGTreeNode, CSGTreeNodeData};


impl<M: Base + Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {

        
    pub fn union_sphere(&mut self, center: V::VectorF, radius: f32, mat: M) -> UnionResult {
        self.union_at_root(&[CSGTreeNode::new_sphere(center, radius, mat)], 0)
    }

    pub fn cut_with_sphere(&mut self, center: V::VectorF, radius: f32, mat: M) -> CutResult {
        self.cut_at_root(&[CSGTreeNode::new_sphere(center, radius, mat)], 0)
    }

    pub fn union_sphere_at_index(&mut self, center: V::VectorF, radius: f32, mat: M, index: CSGTreeIndex) -> UnionResult { 
        self.union_at_index(index, &[CSGTreeNode::new_sphere(center, radius, mat)], 0)
    }

    pub fn cut_with_sphere_at_index(&mut self, center: V::VectorF, radius: f32, mat: M, index: CSGTreeIndex) -> CutResult { 
        self.cut_at_index(index, &[CSGTreeNode::new_sphere(center, radius, mat)], 0)
    }
 
    pub fn union_box_at_index(&mut self, center: V::VectorF, size: V::VectorF, mat: M, index: CSGTreeIndex) -> UnionResult { 
        self.union_at_index(index, &[CSGTreeNode::new_box(center, size, mat)], 0)
    }

}

impl<M: Base + Send + Sync, V: Ve<T, 3>, T: Nu> CSGTree<M, V, T, 3> {
    pub fn add_disk(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M) -> usize {
        self.add_node(CSGTreeNode::new_disk(center, radius, height, mat)) 
    }

    pub fn union_disk(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M) -> UnionResult {
        self.union_at_root(&[CSGTreeNode::new_disk(center, radius, height, mat)], 0)
    }

    pub fn cut_with_disk(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M) -> CutResult {
        self.cut_at_root(&[CSGTreeNode::new_disk(center, radius, height, mat)], 0)
    }

    pub fn union_disk_at_index(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M, index: CSGTreeIndex) -> UnionResult {
        self.union_at_index(index, &[CSGTreeNode::new_disk(center, radius, height, mat)], 0)
    }

    pub fn cut_with_disk_at_index(&mut self, center: V::VectorF, radius: f32, height: f32, mat: M, index: CSGTreeIndex) -> CutResult {
        self.cut_at_index(index, &[CSGTreeNode::new_disk(center, radius, height, mat)], 0)
    }

    pub fn add_shared_grid(&mut self, grid: SharedVoxelGrid) -> usize {
        self.add_node(CSGTreeNode::new_shared_grid(grid)) 
    }

    pub fn union_shared_grid(&mut self, grid: SharedVoxelGrid) -> UnionResult {
        self.union_at_root(&[CSGTreeNode::new_shared_grid(grid)], 0)
    }

    pub fn cut_with_shared_grid(&mut self, grid: SharedVoxelGrid) -> CutResult {
        self.cut_at_root(&[CSGTreeNode::new_shared_grid(grid)], 0)
    }

    pub fn union_shared_grid_at_index(&mut self, grid: SharedVoxelGrid, index: CSGTreeIndex) -> UnionResult {
        self.union_at_index(index, &[CSGTreeNode::new_shared_grid(grid)], 0)
    }

    pub fn remove_shared_grid_at_index(&mut self, grid: SharedVoxelGrid, index: CSGTreeIndex) -> CutResult {
        self.cut_at_index(index, &[CSGTreeNode::new_shared_grid(grid)], 0)
    }
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

impl<M: Base + Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {   
    pub fn union_at_root(&mut self, other: &[CSGTreeNode<M, V, T, D>], other_root: usize) -> UnionResult {   
        if other.is_empty() {
            return UnionResult {
                union_node_index: self.root,
                new_object_index: self.root,
            };
        }

        if self.nodes.is_empty() || matches!(&self.nodes[self.root].data, CSGTreeNodeData::None) {
            self.nodes.extend_from_slice(other);
            self.root = other_root;

            self.calculate_bounds_index(self.root);
            self.changed_bounds = self.get_bounds_index(self.root);

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
            
            //TODO: Handle case where other nodes is also a Union
            union.add_node(new_index);
            self.nodes[new_index].parent = self.root;

            self.index_changed(new_index);
            
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

        self.index_changed(new_index);
        
        UnionResult { 
            union_node_index: union_index, 
            new_object_index: new_index 
        }
    }

    pub fn union_at_index(&mut self, index: CSGTreeIndex, other: &[CSGTreeNode<M, V, T, D>], other_root: usize) -> UnionResult {
        let res = self.union_at_index_internal(index, other, other_root);
        self.index_changed(res.new_object_index);
        res
    }

    fn union_at_index_internal(&mut self, index: CSGTreeIndex, other: &[CSGTreeNode<M, V, T, D>], other_root: usize) -> UnionResult {

        let length = self.nodes.len();
        let new_index = other_root + length;
        self.nodes.extend_from_slice(other);
        shift_node_indecies(&mut self.nodes[length..], length);

        let current_node = &mut self.nodes[index];
        if let CSGTreeNodeData::Union(union) = &mut current_node.data {
            
            //TODO: Handle case where other nodes is also a Union
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

        if self.nodes.is_empty() {
            self.nodes.push(CSGTreeNode::new_none());
        }

        let root_node = &self.nodes[self.root];
        if let CSGTreeNodeData::Cut(cut) = &root_node.data {
            let remove_index = cut.remove;
            let base_index = cut.base;
            let res = self.union_at_index(remove_index, other, other_root);
            self.nodes[res.union_node_index].parent = self.root;

            self.index_changed(res.new_object_index); 

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
        self.nodes.push(CSGTreeNode::new_cut(base_index, new_index));
        
        self.nodes[new_index].parent = cut_index;
        self.nodes[base_index].parent = cut_index;

        self.root = cut_index;

        self.index_changed(new_index);

        CutResult {
            cut_node_index: cut_index,
            base_index,
            new_object_index: new_index,
        }
    }

    pub fn cut_at_index(&mut self, index: CSGTreeIndex, other: &[CSGTreeNode<M, V, T, D>], other_root: usize) -> CutResult {
        self.needs_bounds_recompute = true;

        let current_node = &self.nodes[index];
        if let CSGTreeNodeData::Cut(cut) = &current_node.data {
            let remove_index = cut.remove;
            let base_index = cut.base;
            let res = self.union_at_index(remove_index, other, other_root);
            self.nodes[res.union_node_index].parent = index;

            self.index_changed(res.new_object_index); 

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
        self.nodes.push(CSGTreeNode::new_cut(index, new_index));

        self.update_child(parent, index, cut_index);
        self.nodes[cut_index].parent = parent;
        
        self.nodes[new_index].parent = cut_index;
        self.nodes[index].parent = cut_index;

        self.index_changed(new_index); 
        
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
            CSGTreeNodeData::Cut(remove) => {
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

    fn index_changed(&mut self, index: usize) {
        self.calculate_bounds_index(index);
        self.calculate_bounds_parents(self.nodes[index].parent);

        self.changed_bounds = self.changed_bounds.union(self.get_bounds_index(index));
    }
}

// Can be used to offset the store indecies of nodes so they align when appended.
pub fn shift_node_indecies<M: Base, V: Ve<T, D>, T: Nu, const D: usize>(nodes: &mut [CSGTreeNode<M, V, T, D>], ammount: usize) {
    for node in nodes {
        match &mut node.data {
            CSGTreeNodeData::Union(csgtree_union) => csgtree_union.shift_indecies(ammount),
            CSGTreeNodeData::Cut(csgtree_remove) => csgtree_remove.shift_indecies(ammount),
            _ => {}
        }

        if node.parent != CSG_TREE_INDEX_INVALID {
            node.parent += ammount;
        } 
    }
}

