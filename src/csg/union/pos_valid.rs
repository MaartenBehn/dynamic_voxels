use octa_force::glam::{Vec3A, Vec4};

use crate::{util::{number::Nu, vector::Ve}, volume::VolumeQureyPosValid};

use super::tree::{Union, UnionNodeData};

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValid<V, T, D> for Union<M, V, T, D> {
    fn is_position_valid(&self, pos: V) -> bool {
        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[i];
            if b.aabb.pos_in_aabb(pos) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        UnionNodeData::Box(d) => d.is_position_valid(pos),
                        UnionNodeData::Sphere(d) => d.is_position_valid(pos),
                        UnionNodeData::OffsetVoxelGrid(d) => todo!(),
                        UnionNodeData::SharedVoxelGrid(d) => todo!(),
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
