use octa_force::glam::Mat4;

use crate::{csg::{Base, csg_tree::tree::{CSGTree, CSGTreeNode}, primitves::{CSGPrimitive, PrimitiveType}}, util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{remove::CSGTreeRemove, union::CSGTreeUnion};


/**
These functions do not preserve the structure of the tree or update bounds and changed bounds.
*/
impl<M: Base + Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {  
    pub fn from_node(node: CSGTreeNode<M, V, T, D>) -> Self {
        let mut csg = Self {
            nodes: vec![node],
            needs_bounds_recompute: false,
            root: 0,
            changed_bounds: AABB::default(),
        };

        csg.calculate_bounds_index(0);
        csg.changed_bounds = csg.get_bounds_index(0);

        csg
    }

    pub fn add_primitive<P: PrimitiveType + 'static>(&mut self, p: CSGPrimitive<P, M, V::VectorF, D>) -> usize {
        self.add_node(CSGTreeNode::new_primitive(p))
    }

    pub fn add_sphere(&mut self, center: V::VectorF, radius: f32, mat: M) -> usize {
        self.add_node(CSGTreeNode::new_sphere(CSGPrimitive::new_sphere(center, radius, mat)))
    }

    pub fn add_box(&mut self, center: V::VectorF, size: V::VectorF, mat: M) -> usize {
        self.add_node(CSGTreeNode::new_box(CSGPrimitive::new_box(center, size, mat))) 
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


