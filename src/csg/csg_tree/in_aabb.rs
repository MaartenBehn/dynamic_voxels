use octa_force::glam::vec3;

use crate::{util::{aabb3d::AABB3, iaabb3d::AABBI}, volume::{VolumeQureyAABB, VolumeQureyAABBI, VolumeQureyAABBResult}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::tree::{CSGNodeData, CSGTree, CSGTreeKey};


impl VolumeQureyAABB for CSGTree<u8> {
    fn get_aabb_value(&self, aabb: AABB3) -> VolumeQureyAABBResult {
       self.get_aabb_internal(aabb, self.root_node) 
    }
}

impl VolumeQureyAABBI for CSGTree<u8> {
    fn get_aabb_value_i(&self, aabb: AABBI) -> VolumeQureyAABBResult {
        self.get_aabb_internal(aabb.into(), self.root_node)
    }
}

impl CSGTree<u8> {
    fn get_aabb_internal(&self, aabb: AABB3, index: CSGTreeKey ) -> VolumeQureyAABBResult  {
        let node = &self.nodes[index];

        if !node.aabb.collides_aabb(aabb) {
            return VolumeQureyAABBResult::Full(MATERIAL_ID_NONE);
        }

        match &node.data {
            CSGNodeData::Union(c1, c2) => {
                let a = self.get_aabb_internal(aabb, *c1);
                let b = self.get_aabb_internal(aabb, *c2);

                if matches!(a, VolumeQureyAABBResult::Mixed) || matches!(b, VolumeQureyAABBResult::Mixed) {
                    return VolumeQureyAABBResult::Mixed;
                }

                let a = a.get_value();
                let b = b.get_value();

                if a == b { VolumeQureyAABBResult::Full(a) } 
                else if a == 0 { VolumeQureyAABBResult::Full(b) }
                else if b == 0 { VolumeQureyAABBResult::Full(a) }
                else { VolumeQureyAABBResult::Mixed }
            }
            CSGNodeData::Remove(c1, c2) => {
                let a = self.get_aabb_internal(aabb, *c1);
                let b = self.get_aabb_internal(aabb, *c2);

                if matches!(a, VolumeQureyAABBResult::Mixed) {
                    if matches!(b, VolumeQureyAABBResult::Mixed) {
                        return VolumeQureyAABBResult::Mixed;
                    } else if b.get_value() != 0 {
                        return VolumeQureyAABBResult::Full(MATERIAL_ID_NONE);
                    } else {
                        return VolumeQureyAABBResult::Mixed
                    }
                }

                let a = a.get_value();
                if a == 0 {
                    return VolumeQureyAABBResult::Full(MATERIAL_ID_NONE);
                }

                if matches!(b, VolumeQureyAABBResult::Mixed) {
                    return VolumeQureyAABBResult::Mixed;
                }

                let b = b.get_value();
                if b != 0 { VolumeQureyAABBResult::Full(MATERIAL_ID_NONE) }
                else { VolumeQureyAABBResult::Full(a) }
            }
            CSGNodeData::Intersect(c1, c2) => {
                let a = self.get_aabb_internal(aabb, *c1);
                let b = self.get_aabb_internal(aabb, *c2);

                if matches!(a, VolumeQureyAABBResult::Mixed) || matches!(b, VolumeQureyAABBResult::Mixed) {
                    return VolumeQureyAABBResult::Mixed;
                }

                let a = a.get_value();
                let b = b.get_value();

                if a == 0 || b == 0 { VolumeQureyAABBResult::Full(MATERIAL_ID_NONE) }
                else if a == b { VolumeQureyAABBResult::Full(a) }
                else { VolumeQureyAABBResult::Mixed }
            }
            CSGNodeData::Box(d) => d.get_aabb_value(aabb),
            CSGNodeData::Sphere(d) => d.get_aabb_value(aabb),
            CSGNodeData::All(d) => d.get_aabb_value(aabb),
            CSGNodeData::OffsetVoxelGrid(offset_voxel_grid) => todo!(),
            CSGNodeData::SharedVoxelGrid(shared_voxel_grid) => todo!(),
        }
    }
}
