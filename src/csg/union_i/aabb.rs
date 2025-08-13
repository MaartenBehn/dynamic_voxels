use bvh::{aabb::Bounded, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh, flat_bvh::FlatBvh};
use smallvec::ToSmallVec;

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::{VolumeBounds, VolumeBoundsI}};

use super::tree::{BVHNodeI, CSGUnionI, CSGUnionNodeI, CSGUnionNodeDataI};

impl<T: Send + Sync> VolumeBoundsI for CSGUnionI<T> {
    fn calculate_bounds_i(&mut self) {
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

impl<T> VolumeBoundsI for CSGUnionNodeI<T> {
    fn calculate_bounds_i(&mut self) {
        match &mut self.data {
            CSGUnionNodeDataI::Box(d) => d.calculate_bounds_i(),
            CSGUnionNodeDataI::Sphere(d) => d.calculate_bounds_i(),
            CSGUnionNodeDataI::OffsetVoxelGrid(d) => d.calculate_bounds_i(),
            CSGUnionNodeDataI::SharedVoxelGrid(d) => d.calculate_bounds_i(),
        }
    }

    fn get_bounds_i(&self) -> AABBI {
        match &self.data {
            CSGUnionNodeDataI::Box(d) => d.get_bounds_i(),
            CSGUnionNodeDataI::Sphere(d) => d.get_bounds_i(),
            CSGUnionNodeDataI::OffsetVoxelGrid(d) => d.get_bounds_i(),
            CSGUnionNodeDataI::SharedVoxelGrid(d) => d.get_bounds_i(),
        }
    }
}

impl<T: Send + Sync> CSGUnionI<T> {
    pub fn update_bounds(&mut self) {
        if !self.changed {
            return;
        }
        self.changed = false;

        let bvh = Bvh::build_par(&mut self.nodes);
        let flat_bvh = bvh.flatten_custom(&|aabb, index, exit, shape| {
            let leaf = shape != u32::MAX;

            if leaf {
                let aabb: AABB = self.nodes[shape as usize].get_bounds_i().into();
                BVHNodeI {
                    aabb: aabb.into(),
                    exit: exit as _,
                    leaf: Some(shape as _),
                }
            } else {
                let aabb: AABB = aabb.into();
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
        let aabb: AABB = self.get_bounds_i().into();
        aabb.into()
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
