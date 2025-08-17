use nalgebra::Point;
use octa_force::glam::{vec3, IVec3, UVec3, Vec3A, Vec4};

use crate::{csg::union::tree::UnionNodeData, util::{math::vec3a_to_nalgebra_point, math_config::MC}, volume::{VolumeQureyPosValue}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{new, tree::Union};

impl<C: MC<3>> VolumeQureyPosValue<C::Vector, C::Number, 3> for Union<u8, C, 3> {
    fn get_value(&self, pos: C::Vector) -> u8 {

        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[i];
            if b.aabb.pos_in_aabb(pos) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        UnionNodeData::Box(d) => d.get_value(pos),
                        UnionNodeData::Sphere(d) => d.get_value(pos),
                        UnionNodeData::OffsetVoxelGrid(d) => d.get_value(pos),
                        UnionNodeData::SharedVoxelGrid(d) => d.get_value(pos),
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
