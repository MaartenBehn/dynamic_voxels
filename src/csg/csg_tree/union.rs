use crate::{util::{aabb::AABB, math_config::MC}, volume::VolumeQureyPosValid};

use super::tree::{CSGTreeNode, CSGTreeNodeData, CSGTree, CSGTreeIndex};

#[derive(Debug, Clone, Copy)]
pub struct CSGUnionNode<C: MC<D>, const D: usize> {
    pub aabb: AABB<C::Vector, C::Number, D>,
    pub index: CSGTreeIndex,
    pub bh_index: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct BVHNodeV2<C: MC<D>, const D: usize> {
    pub aabb: AABB<C::Vector, C::Number, D>,
    pub exit: usize,
    pub leaf: Option<CSGTreeIndex>,
}

#[derive(Debug, Clone, Default)]
pub struct CSGTreeUnion<C: MC<D>, const D: usize> {
    pub nodes: Vec<CSGUnionNode<C, D>>,
    pub flat_bvh: Vec<BVHNodeV2<C, D>>,
}





