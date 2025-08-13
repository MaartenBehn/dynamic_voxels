use crate::{util::{iaabb3d::AABBI}, volume::{VolumeQureyAABBI, VolumeQureyAABBResult}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::tree::{CSGUnionI, CSGUnionNodeDataI};

impl VolumeQureyAABBI for CSGUnionI<u8> {
    fn get_aabb_value_i(&self, aabb: AABBI) -> VolumeQureyAABBResult {
        let mut i = 0;
        while i < self.bvh.len() {
            let b = &self.bvh[i];
            if b.aabb.contains_aabb(aabb) {
                if let Some(leaf) = b.leaf {
                    let node = &self.nodes[leaf];
                    let v = match &node.data {
                        CSGUnionNodeDataI::Box(d) => d.get_aabb_value_i(aabb),
                        CSGUnionNodeDataI::Sphere(d) => d.get_aabb_value_i(aabb),
                        CSGUnionNodeDataI::OffsetVoxelGrid(d) => todo!(),
                        CSGUnionNodeDataI::SharedVoxelGrid(d) => todo!(),
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
