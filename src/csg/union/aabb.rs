use bvh::{aabb::Bounded, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh, flat_bvh::FlatBvh};
use octa_force::egui::emath::Numeric;
use smallvec::ToSmallVec;

use crate::{util::{aabb::AABB, aabb3d::AABB3, iaabb3d::AABBI, math_config::MC}, volume::VolumeBounds};

use super::tree::{BVHNode, CSGUnion, CSGUnionNode, CSGUnionNodeData};

impl<V: Send + Sync, C: MC<D>, const D: usize> VolumeBounds<C, D> for CSGUnion<V, C, D> {
    fn calculate_bounds(&mut self) {
        self.update_bounds();
    }

    fn get_bounds(&self) -> AABB<C, D> {
        if self.bvh.is_empty() {
            return AABB::default()
        }

        self.bvh[0].aabb.into()
    }
}

impl<V: Send + Sync, C: MC<D>, const D: usize> VolumeBounds<C, D> for CSGUnionNode<V, C, D> {
    fn calculate_bounds(&mut self) {
        match &mut self.data {
            CSGUnionNodeData::Box(d) => d.calculate_bounds(),
            CSGUnionNodeData::Sphere(d) => d.calculate_bounds(),
            CSGUnionNodeData::OffsetVoxelGrid(d) => d.calculate_bounds(),
            CSGUnionNodeData::SharedVoxelGrid(d) => d.calculate_bounds(),
        }
    }

    fn get_bounds(&self) -> AABB<C, D> {
        match &self.data {
            CSGUnionNodeData::Box(d) => d.get_bounds(),
            CSGUnionNodeData::Sphere(d) => d.get_bounds(),
            CSGUnionNodeData::OffsetVoxelGrid(d) => d.get_bounds(),
            CSGUnionNodeData::SharedVoxelGrid(d) => d.get_bounds(),
        }
    }
}

impl<V: Send + Sync, C: MC<D>, const D: usize> CSGUnion<V, C, D> {
    pub fn update_bounds(&mut self) {
        if !self.changed {
            return;
        }
        self.changed = false;

        let bvh = Bvh::build_par(&mut self.nodes);
        let flat_bvh = bvh.flatten_custom(&|aabb, index, exit, shape| {
            let leaf = shape != u32::MAX;

            if leaf {
                let aabb: AABB<C, D> = self.nodes[shape as usize].get_bounds().into();
                BVHNode {
                    aabb: aabb,
                    exit: exit as _,
                    leaf: Some(shape as _),
                }
            } else {
                 BVHNode {
                    aabb: aabb.into(),
                    exit: exit as _,
                    leaf: None,
                }
            } 
        });

        self.bvh = flat_bvh;
    }
}

impl<V: Send + Sync, C: MC<D>, const D: usize> Bounded<f32, D> for CSGUnionNode<V, C, D> {
    fn aabb(&self) -> bvh::aabb::Aabb<f32, D> {
        self.get_bounds().into()
    }
}

impl<V: Send + Sync, C: MC<D>, const D: usize> BHShape<f32, D> for CSGUnionNode<V, C, D> {
    fn set_bh_node_index(&mut self, i: usize) {
        self.bh_index = i;
    }

    fn bh_node_index(&self) -> usize {
        self.bh_index
    }
}
