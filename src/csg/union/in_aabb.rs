use crate::{util::aabb3d::AABB, volume::{VolumeQureyAABB, VolumeQureyAABBResult}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::tree::{CSGUnion, CSGUnionNodeData};

impl VolumeQureyAABB for CSGUnion<u8> {
    fn get_aabb_value(&self, aabb: AABB) -> VolumeQureyAABBResult {
        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[0];
            if b.aabb.contains_aabb(aabb) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        CSGUnionNodeData::Box(d) => d.get_aabb_value(aabb),
                        CSGUnionNodeData::Sphere(d) => d.get_aabb_value(aabb),
                        CSGUnionNodeData::OffsetVoxelGrid(d) => todo!(),
                        CSGUnionNodeData::SharedVoxelGrid(d) => todo!(),
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
