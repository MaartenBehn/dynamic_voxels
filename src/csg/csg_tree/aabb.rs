use std::mem;

use itertools::Itertools;
use octa_force::{egui::emath::Numeric, glam::Vec3A};
use smallvec::ToSmallVec;

use crate::{bvh::Bvh, util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, volume::VolumeBounds, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::{tree::{CSGTreeNode, CSGTreeNodeData, CSGTree, CSGTreeIndex}, union::{BVHNodeCSGUnion, CSGTreeUnion}};

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for CSGTree<M, V, T, D> {
    fn calculate_bounds(&mut self) {
        if !self.needs_bounds_recompute {
            return;
        }
        self.needs_bounds_recompute = false;

        self.calculate_bounds_index(self.root);
    }

    fn get_bounds(&self) -> AABB<V, T, D> {
        if self.nodes.is_empty() {
            return AABB::default();
        }

        let aabb = self.get_bounds_index(self.root);

        aabb
    }
}

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {
    pub fn calculate_bounds_index(&mut self, index: CSGTreeIndex) {
        let node = &mut self.nodes[index];

        match &mut node.data {
            CSGTreeNodeData::None => {},
            CSGTreeNodeData::Union(d) => {
                if !d.needs_bounds_recompute {
                    return;
                }
                d.needs_bounds_recompute = false;

                let mut union = mem::take(d);
                
                for index in union.indecies.iter() {
                    self.calculate_bounds_index(*index);
                }

                union.bvh = Bvh::<BVHNodeCSGUnion<V, T, D>, (), V::VectorF, f32, D>::build_par(
                    &self.nodes, 
                    &mut union.indecies);

                self.nodes[index].data = CSGTreeNodeData::Union(union);
            },
            CSGTreeNodeData::Cut(csgtree_remove) => {
                let base = csgtree_remove.base;
                let remove = csgtree_remove.remove;
                self.calculate_bounds_index(base);
                self.calculate_bounds_index(remove);
            },
            CSGTreeNodeData::Box(d) => d.calculate_bounds(),
            CSGTreeNodeData::Sphere(d) => d.calculate_bounds(),
            CSGTreeNodeData::OffsetVoxelGrid(d) => 
            <OffsetVoxelGrid as VolumeBounds<V, T, D>>::calculate_bounds(d),
            CSGTreeNodeData::SharedVoxelGrid(d) => 
            <SharedVoxelGrid as VolumeBounds<V, T, D>>::calculate_bounds(d),
        }
    }

    pub fn calculate_bounds_parents(&mut self, index: CSGTreeIndex) {
        let node = &mut self.nodes[index];

        match &mut node.data {
            CSGTreeNodeData::Union(d) => {
                if !d.needs_bounds_recompute {
                    return;
                }
                d.needs_bounds_recompute = false;

                let mut union = mem::take(d);
                
                union.bvh = Bvh::<BVHNodeCSGUnion<V, T, D>, (), V::VectorF, f32, D>::build_par(
                    &self.nodes, 
                    &mut union.indecies);

                self.nodes[index].data = CSGTreeNodeData::Union(union);
            },
            CSGTreeNodeData::Cut(csgtree_remove) => {},
            CSGTreeNodeData::None
            | CSGTreeNodeData::Box(_)
            | CSGTreeNodeData::Sphere(_)
            | CSGTreeNodeData::OffsetVoxelGrid(_) 
            | CSGTreeNodeData::SharedVoxelGrid(_) => unreachable!()
        }

        if index != self.root {
            self.calculate_bounds_parents(self.nodes[index].parent);
        }
    }

    pub fn get_bounds_index(&self, index: CSGTreeIndex) -> AABB<V, T, D> {
        let node = &self.nodes[index];

        match &node.data {
            CSGTreeNodeData::None => AABB::default(),
            CSGTreeNodeData::Union(d) => d.get_bounds(),
            CSGTreeNodeData::Cut(csgtree_remove) => {
                let base = csgtree_remove.base;
                self.get_bounds_index(base)
            },
            CSGTreeNodeData::Box(d) => d.get_bounds(),
            CSGTreeNodeData::Sphere(d) => d.get_bounds(),
            CSGTreeNodeData::OffsetVoxelGrid(d) => d.get_bounds(),
            CSGTreeNodeData::SharedVoxelGrid(d) => d.get_bounds(),
        }
    }

    fn calculate_bounds_union(&mut self, union: &mut CSGTreeUnion<V, T, D>) {

            }
}



impl<V: Ve<T, D>, T: Nu, const D: usize> CSGTreeUnion<V, T, D> {
    pub fn get_bounds(&self) -> AABB<V, T, D> {
        if self.bvh.nodes.is_empty() {
            return AABB::default();
        }

        self.bvh.nodes[0].aabb
    }
}


