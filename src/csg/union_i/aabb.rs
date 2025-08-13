use bvh::{aabb::Bounded, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh, flat_bvh::FlatBvh};
use smallvec::ToSmallVec;

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::{VolumeBounds, VolumeBoundsI}};

use super::tree::{BVHNodeI, CSGUnionI, CSGUnionNodeI, CSGUnionNodeDataI};

impl<T: Send + Sync> VolumeBoundsI for CSGUnionI<T> {
    fn calculate_bounds(&mut self) {
        self.update_bounds();
    }

    fn get_bounds_i(&self) -> AABBI {
        if self.bvh.is_empty() {
            return AABBI::default()
        }

        let aabb: AABB = self.bvh[0].aabb.into();
        aabb.into()
    }
}

impl<T: Send + Sync> CSGUnionI<T> {
    pub fn update_bounds(&mut self) {
        if !self.changed {
            return;
        }

        let bvh = Bvh::build_par(&mut self.nodes);
        let flat_bvh = bvh.flatten_custom(&|aabb, index, exit, shape| {
            let leaf = shape != u32::MAX;
            let aabb: AABB = aabb.into();

            if leaf {
                BVHNodeI {
                    aabb: aabb.into(),
                    exit: exit as _,
                    leaf: Some(shape as _),
                }
            } else {
                 BVHNodeI {
                    aabb: aabb.into(),
                    exit: exit as _,
                    leaf: None,
                }
            } 
        });

        self.bvh = flat_bvh;
    }
}

impl<T> Bounded<f32, 3> for CSGUnionNodeI<T> {
    fn aabb(&self) -> bvh::aabb::Aabb<f32, 3> {
        match &self.data {
            CSGUnionNodeDataI::Box(d) => d.get_bounds(),
            CSGUnionNodeDataI::Sphere(d) => d.get_bounds(),
            CSGUnionNodeDataI::OffsetVoxelGrid(d) => d.get_bounds(),
            CSGUnionNodeDataI::SharedVoxelGrid(d) => d.get_bounds(),
        }.into()
    }
}

impl<T> BHShape<f32, 3> for CSGUnionNodeI<T> {
    fn set_bh_node_index(&mut self, i: usize) {
        self.bh_index = i;
    }

    fn bh_node_index(&self) -> usize {
        self.bh_index
    }
}
