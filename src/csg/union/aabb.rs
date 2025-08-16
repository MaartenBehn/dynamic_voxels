use bvh::{aabb::Bounded, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh, flat_bvh::FlatBvh};
use octa_force::egui::emath::Numeric;
use smallvec::ToSmallVec;

use crate::{util::{aabb::AABB, math_config::MC}, volume::VolumeBounds, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::tree::{BVHNode, CSGUnion, CSGUnionNode, CSGUnionNodeData};

impl<V: Send + Sync, C: MC<D>, const D: usize> VolumeBounds<C::Vector, C::Number, D> for CSGUnion<V, C, D> {
    fn calculate_bounds(&mut self) {
        self.update_bounds();
    }

    fn get_bounds(&self) -> AABB<C::Vector, C::Number, D> {
        if self.bvh.is_empty() {
            return AABB::default()
        }

        self.bvh[0].aabb.into()
    }
}

impl<V: Send + Sync, C: MC<D>, const D: usize> VolumeBounds<C::Vector, C::Number, D> for CSGUnionNode<V, C, D> {
    fn calculate_bounds(&mut self) {
        match &mut self.data {
            CSGUnionNodeData::Box(d) => d.calculate_bounds(),
            CSGUnionNodeData::Sphere(d) => d.calculate_bounds(),
            CSGUnionNodeData::OffsetVoxelGrid(d) => 
                <OffsetVoxelGrid as VolumeBounds<C::Vector, C::Number, D>>::calculate_bounds(d),
            CSGUnionNodeData::SharedVoxelGrid(d) => 
                <SharedVoxelGrid as VolumeBounds<C::Vector, C::Number, D>>::calculate_bounds(d),
        }
    }

    fn get_bounds(&self) -> AABB<C::Vector, C::Number, D> {
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
                let aabb = self.nodes[shape as usize].get_bounds();

                BVHNode {
                    aabb: aabb,
                    exit: exit as _,
                    leaf: Some(shape as _),
                }
            } else {
                let aabb: AABB<C::VectorF, f32, D> = aabb.into();
                let aabb = AABB::<C::Vector, C::Number, D>::from_f(aabb);

                BVHNode {
                    aabb: aabb,
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
        self.get_bounds().to_f::<C::VectorF>().into()
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
