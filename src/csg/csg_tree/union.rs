use crate::{bvh::{node::BHNode, shape::{BHShape, Shapes}, Bvh}, util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::tree::{CSGTreeNode, CSGTreeNodeData, CSGTree, CSGTreeIndex};


#[derive(Debug, Clone, Copy, Default)]
pub struct BVHNodeV2<V: Ve<T, D>, T: Nu, const D: usize> {
    pub aabb: AABB<V, T, D>,
    pub exit: usize,
    pub leaf: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct CSGTreeUnion<V: Ve<T, D>, T: Nu, const D: usize> {
    pub indecies: Vec<CSGTreeIndex>,
    pub bvh: Bvh<BVHNodeV2<V, T, D>, V::VectorF, f32, D>,
    pub changed: bool,
}

impl<V: Ve<T, D>, T: Nu, const D: usize> CSGTreeUnion<V, T, D> {
    pub fn new(indecies: Vec<CSGTreeIndex>) -> Self { 
        Self {
            indecies,
            bvh: Bvh::default(),
            changed: true,
        }
    }

    pub fn add_node(&mut self, index: CSGTreeIndex) {
        self.indecies.push(index);
        self.changed = true;
    }

    pub fn shift_indecies(&mut self, ammount: usize) {
        for index in self.indecies.iter_mut() {
            *(index) += ammount;
        }
    }
} 

impl<V: Ve<T, D>, T: Nu, const D: usize> BHNode<V::VectorF, f32, D> for BVHNodeV2<V, T, D> {
    fn new(aabb: AABB<V::VectorF, f32, D>, exit_index: usize, shape_index: Option<usize>) -> Self {
        Self {
            aabb: AABB::from_f(aabb),
            exit: exit_index,
            leaf: shape_index,
        }
    }
}

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> BHShape<V::VectorF, f32, D> for CSGTreeNode<M, V, T, D> {
    fn aabb(&self, shapes: &Shapes<Self, V::VectorF, f32, D>) -> AABB<V::VectorF, f32, D> {
        match &self.data {
            CSGTreeNodeData::None => AABB::default(),
            CSGTreeNodeData::Union(d) => d.get_bounds().to_f(),
            CSGTreeNodeData::Cut(csgtree_remove) => {
                let base = csgtree_remove.base;
                shapes.aabb(base)
            },
            CSGTreeNodeData::Box(d) => d.get_bounds().to_f(),
            CSGTreeNodeData::Sphere(d) => d.get_bounds().to_f(),
            CSGTreeNodeData::OffsetVoxelGrid(d) => d.get_bounds(),
            CSGTreeNodeData::SharedVoxelGrid(d) => d.get_bounds(),
        }
    }
}
