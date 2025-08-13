use nalgebra::Point;
use octa_force::glam::{vec3, IVec3, UVec3, Vec3A, Vec4};

use crate::{csg::union::tree::CSGUnionNodeData, util::{aabb3d::AABB, math::vec3a_to_nalgebra_point}, volume::{VolumeQureyPosValue, VolumeQureyPosValueI}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{new, tree::CSGUnion};

impl VolumeQureyPosValue for CSGUnion<u8> {
    fn get_value(&self, posa: Vec3A) -> u8 {
        let pos = Vec4::from((posa, 1.0));

        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[0];
            if b.aabb.pos_in_aabb(pos) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        CSGUnionNodeData::Box(d) => d.get_value(posa),
                        CSGUnionNodeData::Sphere(d) => d.get_value(posa),
                        CSGUnionNodeData::OffsetVoxelGrid(d) => d.get_value(posa),
                        CSGUnionNodeData::SharedVoxelGrid(d) => d.get_value(posa),
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
