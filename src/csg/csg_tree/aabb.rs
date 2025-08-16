use std::mem;

use bvh::{aabb::Bounded, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh, flat_bvh::FlatBvh};
use octa_force::egui::emath::Numeric;
use smallvec::ToSmallVec;

use crate::{util::{aabb::AABB, math_config::MC}, volume::VolumeBounds, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{tree::{CSGTreeNode, CSGTreeNodeData, CSGTree, CSGTreeIndex}, union::{BVHNodeV2, CSGTreeUnion, CSGUnionNode}};

impl<V: Send + Sync, C: MC<D>, const D: usize> VolumeBounds<C::Vector, C::Number, D> for CSGTree<V, C, D> {
    fn calculate_bounds(&mut self) {
        self.calculate_bounds_index(self.root);
    }

    fn get_bounds(&self) -> AABB<C::Vector, C::Number, D> {
        self.get_bounds_index(self.root)
    }
}

impl<V: Send + Sync, C: MC<D>, const D: usize> CSGTree<V, C, D> {
    fn calculate_bounds_index(&mut self, index: CSGTreeIndex) {
        let node = &mut self.nodes[index];

        match &mut node.data {
            CSGTreeNodeData::Union(d) => {
                        let mut union = mem::take(d);
                        self.calculate_bounds_union(&mut union);
                        self.nodes[index].data = CSGTreeNodeData::Union(union);
                    },
            CSGTreeNodeData::Remove(csgtree_remove) => {
                let base = csgtree_remove.base;
                self.calculate_bounds_index(base);
            },

            CSGTreeNodeData::Box(d) => d.calculate_bounds(),
            CSGTreeNodeData::Sphere(d) => d.calculate_bounds(),
            CSGTreeNodeData::OffsetVoxelGrid(d) => 
                        <OffsetVoxelGrid as VolumeBounds<C::Vector, C::Number, D>>::calculate_bounds(d),
            CSGTreeNodeData::SharedVoxelGrid(d) => 
                        <SharedVoxelGrid as VolumeBounds<C::Vector, C::Number, D>>::calculate_bounds(d),
        }
    }

    fn get_bounds_index(&self, index: CSGTreeIndex) -> AABB<C::Vector, C::Number, D> {
        let node = &self.nodes[index];

        match &node.data {
            CSGTreeNodeData::Union(d) => d.get_bounds(),
            CSGTreeNodeData::Remove(csgtree_remove) => {
                let base = csgtree_remove.base;
                self.get_bounds_index(base)
            },

            CSGTreeNodeData::Box(d) => d.get_bounds(),
            CSGTreeNodeData::Sphere(d) => d.get_bounds(),
            CSGTreeNodeData::OffsetVoxelGrid(d) => d.get_bounds(),
            CSGTreeNodeData::SharedVoxelGrid(d) => d.get_bounds(),
        }
    }

    fn calculate_bounds_union(&mut self, union: &mut CSGTreeUnion<C, D>) {

        for node in union.nodes.iter_mut() {
            self.calculate_bounds_index(node.index);
            node.aabb = self.get_bounds_index(node.index);
        }

        let bvh = Bvh::build_par(&mut union.nodes);
        let flat_bvh = bvh.flatten_custom(&|aabb, index, exit, shape| {
            let leaf = shape != u32::MAX;

            if leaf {
                let aabb = union.nodes[shape as usize].aabb;

                BVHNodeV2 {
                    aabb: aabb,
                    exit: exit as _,
                    leaf: Some(shape as _),
                }
            } else {
                let aabb: AABB<C::VectorF, f32, D> = aabb.into();
                let aabb = AABB::<C::Vector, C::Number, D>::from_f(aabb);

                BVHNodeV2 {
                    aabb: aabb,
                    exit: exit as _,
                    leaf: None,
                }
            } 
        });

        union.flat_bvh = flat_bvh;
    }
}



impl<C: MC<D>, const D: usize> CSGTreeUnion<C, D> {
    fn get_bounds(&self) -> AABB<C::Vector, C::Number, D> {
        if self.flat_bvh.is_empty() {
            return AABB::default();
        }

        self.flat_bvh[0].aabb
    }
}

impl<C: MC<D>, const D: usize> Bounded<f32, D> for CSGUnionNode<C, D> {
    fn aabb(&self) -> bvh::aabb::Aabb<f32, D> {
        self.aabb.to_f::<C::VectorF>().into()
    }
}

impl<C: MC<D>, const D: usize> BHShape<f32, D> for CSGUnionNode<C, D> {
    fn set_bh_node_index(&mut self, i: usize) {
        self.bh_index = i;
    }

    fn bh_node_index(&self) -> usize {
        self.bh_index
    }
}
