use octa_force::glam::Mat4;

use crate::{csg::{Base, r#box::CSGBox, csg_tree::tree::{CSGTree, CSGTreeNode}, sphere::CSGSphere}, util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{remove::CSGTreeRemove, union::CSGTreeUnion};


/**
These functions do not preserve the structure of the tree or update bounds and changed bounds.
*/
impl<M: Base, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {  
    pub fn from_node(node: CSGTreeNode<M, V, T, D>) -> Self {
        Self {
            nodes: vec![node],
            needs_bounds_recompute: true,
            root: 0,
            changed_bounds: AABB::default(),
        }
    }

    pub fn add_sphere(&mut self, center: V::VectorF, radius: f32, mat: M) -> usize {
        self.add_node(CSGTreeNode::new_sphere(center, radius, mat)) 
    }

    pub fn add_box(&mut self, center: V::VectorF, size: V::VectorF, mat: M) -> usize {
        self.add_node(CSGTreeNode::new_box(center, size, mat)) 
    }
        
    pub fn add_node(&mut self, node: CSGTreeNode<M, V, T, D>) -> usize {
        self.needs_bounds_recompute = true;

        let i = self.nodes.len();
        self.nodes.push(node); 
        i
    }

    pub fn add_union_node(&mut self, indecies: Vec<usize>) -> usize {
        self.needs_bounds_recompute = true;

        let i = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_union(indecies)); 
        i
    }

    pub fn add_cut_node(&mut self, base: usize, cut: usize) -> usize {
        self.needs_bounds_recompute = true;

        let i = self.nodes.len();
        self.nodes.push(CSGTreeNode::new_cut(base, cut)); 
        i
    }

    pub fn set_root(&mut self, root: usize) {
        self.needs_bounds_recompute = true;

        self.root = root;
    }
}


