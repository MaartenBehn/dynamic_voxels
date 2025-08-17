use crate::{bvh::{node::BHNode, shape::{BHShape, Shapes}, Bvh}, util::{aabb::AABB, math_config::MC}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::tree::{CSGTreeNode, CSGTreeNodeData, CSGTree, CSGTreeIndex};



#[derive(Debug, Clone, Copy, Default)]
pub struct BVHNodeV2<C: MC<D>, const D: usize> {
    pub aabb: AABB<C::Vector, C::Number, D>,
    pub exit: usize,
    pub leaf: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct CSGTreeUnion<C: MC<D>, const D: usize> {
    pub indecies: Vec<CSGTreeIndex>,
    pub bvh: Bvh<BVHNodeV2<C, D>, C::VectorF, f32, D>,
    pub changed: bool,
}

impl<C: MC<D>, const D: usize> CSGTreeUnion<C, D> {
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
} 

impl<C: MC<D>, const D: usize> BHNode<C::VectorF, f32, D> for BVHNodeV2<C, D> {
    fn new(aabb: AABB<C::VectorF, f32, D>, exit_index: usize, shape_index: Option<usize>) -> Self {
        Self {
            aabb: AABB::from_f(aabb),
            exit: exit_index,
            leaf: shape_index,
        }
    }
}

impl<V: Send + Sync, C: MC<D>, const D: usize> BHShape<C::VectorF, f32, D> for CSGTreeNode<V, C, D> {
    fn aabb(&self, shapes: &Shapes<Self, C::VectorF, f32, D>) -> AABB<C::VectorF, f32, D> {
        match &self.data {
            CSGTreeNodeData::Union(d) => d.get_bounds().to_f(),
            CSGTreeNodeData::Remove(csgtree_remove) => {
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
