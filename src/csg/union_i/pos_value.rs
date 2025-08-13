use nalgebra::Point;
use octa_force::glam::{vec3, IVec3, UVec3, Vec3A, Vec4};

use crate::{util::{aabb3d::AABB, math::vec3a_to_nalgebra_point}, volume::{VolumeQureyPosValue, VolumeQureyPosValueI}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{new, tree::{CSGUnionI, CSGUnionNodeDataI}};

impl VolumeQureyPosValueI for CSGUnionI<u8> {
    fn get_value_i(&self, pos: IVec3) -> u8 {

        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[i];
            if b.aabb.pos_in_aabb(pos) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        CSGUnionNodeDataI::Box(d) => d.get_value_i(pos),
                        CSGUnionNodeDataI::Sphere(d) => d.get_value_i(pos),
                        CSGUnionNodeDataI::OffsetVoxelGrid(d) => d.get_value_i(pos),
                        CSGUnionNodeDataI::SharedVoxelGrid(d) => d.get_value_i(pos),
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

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        unimplemented!()
    }
}
