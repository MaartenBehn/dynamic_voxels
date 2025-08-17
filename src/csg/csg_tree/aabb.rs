use std::mem;

use itertools::Itertools;
use octa_force::{egui::emath::Numeric, glam::Vec3A};
use smallvec::ToSmallVec;

use crate::{bvh::{Bvh}, util::{aabb::AABB, math_config::MC}, volume::VolumeBounds, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{tree::{CSGTreeNode, CSGTreeNodeData, CSGTree, CSGTreeIndex}, union::{BVHNodeV2, CSGTreeUnion}};

impl<V: Send + Sync, C: MC<D>, const D: usize> VolumeBounds<C::Vector, C::Number, D> for CSGTree<V, C, D> {
    fn calculate_bounds(&mut self) {
        if !self.changed {
            return;
        }
        self.changed = false;

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
                if !d.changed {
                    return;
                }
                d.changed = false;

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

        for index in union.indecies.iter() {
            self.calculate_bounds_index(*index);
        }
 
        union.bvh = Bvh::<BVHNodeV2<C, D>, C::VectorF, f32, D>::build_par(
            &self.nodes, 
            &mut union.indecies);
    }
}



impl<C: MC<D>, const D: usize> CSGTreeUnion<C, D> {
    pub fn get_bounds(&self) -> AABB<C::Vector, C::Number, D> {
        if self.bvh.nodes.is_empty() {
            return AABB::default();
        }

        self.bvh.nodes[0].aabb
    }
}


