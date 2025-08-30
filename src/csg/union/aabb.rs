use bvh::{aabb::Bounded, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh, flat_bvh::FlatBvh};
use octa_force::egui::emath::Numeric;
use smallvec::ToSmallVec;

use crate::{util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, volume::VolumeBounds, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid}};

use super::tree::{BVHNode, Union, UnionNode, UnionNodeData};

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for Union<M, V, T, D> {
    fn calculate_bounds(&mut self) {
        self.update_bounds();
    }

    fn get_bounds(&self) -> AABB<V, T, D> {
        if self.bvh.is_empty() {
            return AABB::default()
        }

        self.bvh[0].aabb.into()
    }
}

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> VolumeBounds<V, T, D> for UnionNode<M, V, T, D> {
    fn calculate_bounds(&mut self) {
        match &mut self.data {
            UnionNodeData::Box(d) => d.calculate_bounds(),
            UnionNodeData::Sphere(d) => d.calculate_bounds(),
            UnionNodeData::OffsetVoxelGrid(d) => 
                <OffsetVoxelGrid as VolumeBounds<V, T, D>>::calculate_bounds(d),
            UnionNodeData::SharedVoxelGrid(d) => 
                <SharedVoxelGrid as VolumeBounds<V, T, D>>::calculate_bounds(d),
        }
    }

    fn get_bounds(&self) -> AABB<V, T, D> {
        match &self.data {
            UnionNodeData::Box(d) => d.get_bounds(),
            UnionNodeData::Sphere(d) => d.get_bounds(),
            UnionNodeData::OffsetVoxelGrid(d) => d.get_bounds(),
            UnionNodeData::SharedVoxelGrid(d) => d.get_bounds(),
        }
    }
}

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> Union<M, V, T, D> {
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
                let aabb: AABB<V::VectorF, f32, D> = aabb.into();
                let aabb = AABB::<V, T, D>::from_f(aabb);

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

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> Bounded<f32, D> for UnionNode<M, V, T, D> {
    fn aabb(&self) -> bvh::aabb::Aabb<f32, D> {
        self.get_bounds().to_f::<V::VectorF>().into()
    }
}

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> BHShape<f32, D> for UnionNode<M, V, T, D> {
    fn set_bh_node_index(&mut self, i: usize) {
        self.bh_index = i;
    }

    fn bh_node_index(&self) -> usize {
        self.bh_index
    }
}
