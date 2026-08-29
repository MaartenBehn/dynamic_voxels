use std::fmt::Debug;

use crate::{bvh::{Bvh, node::BHNode, shape::{BHShape, Shapes}}, csg::primitves::{CSGPrimitive, r#box::CSGBox, cylinder::CSGCylinder, sphere::CSGSphere}, util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::tree::{CSGTreeNode, CSGTreeNodeData, CSGTree, CSGTreeIndex};


#[derive(Debug, Clone, Copy, Default)]
pub struct BVHNodeCSGIntersect<V: Ve<T, D>, T: Nu, const D: usize> {
    pub aabb: AABB<V, T, D>,
    pub exit: usize,
    pub leaf: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct CSGTreeIntersect<V: Ve<T, D>, T: Nu, const D: usize> {
    pub indecies: Vec<CSGTreeIndex>,
    pub aabb: AABB<V, T, D>,
    pub needs_bounds_recompute: bool,
}

impl<V: Ve<T, D>, T: Nu, const D: usize> CSGTreeIntersect<V, T, D> {
    pub fn new(indecies: Vec<CSGTreeIndex>) -> Self { 
        Self {
            indecies,
            aabb: AABB::default(),
            needs_bounds_recompute: true,
        }
    }

    pub fn add_node(&mut self, index: CSGTreeIndex) {
        self.indecies.push(index);
        self.needs_bounds_recompute = true;
    }

    pub fn shift_indecies(&mut self, ammount: usize) {
        for index in self.indecies.iter_mut() {
            *(index) += ammount;
        }
    }
} 

impl<V: Ve<T, D>, T: Nu, const D: usize> Default for CSGTreeIntersect<V, T, D> {
    fn default() -> Self {
        Self { 
            indecies: Default::default(), 
            aabb: AABB::default(), 
            needs_bounds_recompute: Default::default() 
        }
    }
}
