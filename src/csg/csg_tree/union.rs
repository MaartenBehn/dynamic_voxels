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
    pub changed: bool,
}

impl<C: MC<D>, const D: usize> CSGTreeUnion<C, D> {
    pub fn new(nodes: Vec<CSGTreeIndex>) -> Self {
        let nodes = nodes.iter()
            .map(|&index| {
                CSGUnionNode {
                    aabb: AABB::default(),
                    index,
                    bh_index: 0,
                }
            })
            .collect();

        Self {
            nodes,
            flat_bvh: vec![],
            changed: true,
        }
    }

    pub fn add_node(&mut self, index: CSGTreeIndex) {
        self.nodes.push(CSGUnionNode { 
            aabb: AABB::default(), 
            index, 
            bh_index: 0 
        });
        self.changed = true;
    }
} 
