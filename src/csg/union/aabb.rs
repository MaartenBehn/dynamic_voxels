use bvh::{aabb::Bounded, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh, flat_bvh::FlatBvh};
use smallvec::ToSmallVec;

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::{VolumeBounds, VolumeBoundsI}};

use super::tree::{BVHNode, CSGUnion, CSGUnionNode, CSGUnionNodeData};

impl<T: Send + Sync> VolumeBounds for CSGUnion<T> {
    fn calculate_bounds(&mut self) {
        self.update_bounds();
    }

    fn get_bounds(&self) -> AABB {
        if self.bvh.is_empty() {
            return AABB::default()
        }

        self.bvh[0].aabb.into()
    }
}

impl<T: Send + Sync> VolumeBoundsI for CSGUnion<T> {
    fn calculate_bounds(&mut self) {
        self.update_bounds();
    }

    fn get_bounds_i(&self) -> AABBI {
        self.get_bounds().into()
    }
}

impl<T: Send + Sync> CSGUnion<T> {
    pub fn update_bounds(&mut self) {
        if !self.changed {
            return;
        }

        let bvh = Bvh::build_par(&mut self.nodes);
        let flat_bvh = bvh.flatten_custom(&|aabb, index, exit, shape| {
            let leaf = shape != u32::MAX;

            if leaf {
                BVHNode {
                    aabb: aabb.into(),
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

impl<T> Bounded<f32, 3> for CSGUnionNode<T> {
    fn aabb(&self) -> bvh::aabb::Aabb<f32, 3> {
        match &self.data {
            CSGUnionNodeData::Box(d) => d.get_bounds(),
            CSGUnionNodeData::Sphere(d) => d.get_bounds(),
            CSGUnionNodeData::OffsetVoxelGrid(d) => d.get_bounds(),
            CSGUnionNodeData::SharedVoxelGrid(d) => d.get_bounds(),
        }.into()
    }
}

impl<T> BHShape<f32, 3> for CSGUnionNode<T> {
    fn set_bh_node_index(&mut self, i: usize) {
        self.bh_index = i;
    }

    fn bh_node_index(&self) -> usize {
        self.bh_index
    }
}
