use octa_force::glam::{Vec3A, Vec4};

use crate::{util::math_config::MC, volume::VolumeQureyPosValid};

use super::tree::{CSGUnion, CSGUnionNodeData};

impl<V: Send + Sync, C: MC<D>, const D: usize> VolumeQureyPosValid<C::Vector, C::Number, D> for CSGUnion<V, C, D> {
    fn is_position_valid(&self, pos: C::Vector) -> bool {
        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[i];
            if b.aabb.pos_in_aabb(pos) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        CSGUnionNodeData::Box(d) => d.is_position_valid(pos),
                        CSGUnionNodeData::Sphere(d) => d.is_position_valid(pos),
                        CSGUnionNodeData::OffsetVoxelGrid(d) => todo!(),
                        CSGUnionNodeData::SharedVoxelGrid(d) => todo!(),
                    };

                    if v {
                        return true;
                    }
                }

                i += 1;
            } else {
                i = b.exit;
            }
        }

        false
    }
}
