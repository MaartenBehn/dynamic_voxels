use crate::{util::{aabb::AABB, math_config::MC, number::Nu, vector::Ve}, volume::{VolumeQureyAABB, VolumeQureyAABBResult}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::tree::{Union, UnionNodeData};

impl<V: Ve<T, 3>, T: Nu> VolumeQureyAABB<V, T, 3> for Union<u8, V, T, 3> {
    fn get_aabb_value(&self, aabb: AABB<V, T, 3>) -> VolumeQureyAABBResult {
        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[i];
            if b.aabb.collides_aabb(aabb) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        UnionNodeData::Box(d) => d.get_aabb_value(aabb),
                        UnionNodeData::Sphere(d) => d.get_aabb_value(aabb),
                        UnionNodeData::OffsetVoxelGrid(d) => todo!(),
                        UnionNodeData::SharedVoxelGrid(d) => todo!(),
                    };

                    if !matches!(v, VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)) {
                        return v;
                    }
                }

                i += 1;
            } else {
                i = b.exit;
            }
        }

        VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
    }
}
