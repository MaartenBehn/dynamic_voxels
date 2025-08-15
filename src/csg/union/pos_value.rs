use nalgebra::Point;
use octa_force::glam::{vec3, IVec3, UVec3, Vec3A, Vec4};

use crate::{csg::union::tree::CSGUnionNodeData, util::{aabb3d::AABB3, math::vec3a_to_nalgebra_point, math_config::MC}, volume::{VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{new, tree::CSGUnion};

impl<C: MC<D>, const D: usize> VolumeQureyPosValue<C, D> for CSGUnion<u8, C, D> {
    fn get_value(&self, pos: C::Vector) -> u8 {

        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[i];
            if b.aabb.pos_in_aabb(pos) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        CSGUnionNodeData::Box(d) => d.get_value(pos),
                        CSGUnionNodeData::Sphere(d) => d.get_value(pos),
                        CSGUnionNodeData::OffsetVoxelGrid(d) => d.get_value(pos),
                        CSGUnionNodeData::SharedVoxelGrid(d) => d.get_value(pos),
                    };

                    if v != MATERIAL_ID_NONE {
                        return v;
                    }
                }

                i += 1;
            } else {
                i = b.exit;
            }
        }

        MATERIAL_ID_NONE
    }
}
