use octa_force::glam::vec3;

use crate::{util::aabb::AABB, volume::{VolumeQureyAABB, VolumeQureyAABBResult}, voxel::renderer::palette::MATERIAL_ID_NONE};

use super::tree::{SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};



impl VolumeQureyAABB for SlotMapCSGTree<u8> {
    fn get_aabb_value(&self, aabb: AABB) -> VolumeQureyAABBResult {
       self.get_aabb_internal(aabb, self.root_node) 
    }
}

impl SlotMapCSGTree<u8> {
    fn get_aabb_internal(&self, aabb: AABB, index: SlotMapCSGTreeKey ) -> VolumeQureyAABBResult  {
        let node = &self.nodes[index];

        if !node.aabb.collides_aabb(aabb) {
            return VolumeQureyAABBResult::Full(MATERIAL_ID_NONE);
        }

        match &node.data {
            SlotMapCSGNodeData::Union(c1, c2) => {
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
            SlotMapCSGNodeData::Remove(c1, c2) => {
                let a = self.get_aabb_internal(aabb, *c1);
                let b = self.get_aabb_internal(aabb, *c2);

                if matches!(a, VolumeQureyAABBResult::Mixed) {
                    if matches!(b, VolumeQureyAABBResult::Mixed) {
                        return VolumeQureyAABBResult::Mixed;
                    } else if b.get_value() != 0 {
                        return VolumeQureyAABBResult::Full(MATERIAL_ID_NONE);
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
            SlotMapCSGNodeData::Mat(mat, c) => {
                let aabb = aabb.mul_mat(mat);
                self.get_aabb_internal(aabb, *c)
            }
            SlotMapCSGNodeData::Intersect(c1, c2) => {
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
            SlotMapCSGNodeData::Box(mat, v) => {
                let aabb = aabb.mul_mat(mat);

                let b = AABB::new(
                    vec3(-0.5, -0.5, -0.5), 
                    vec3(0.5, 0.5, 0.5));

                if aabb.contains_aabb(b) {
                    VolumeQureyAABBResult::Full(*v)
                } else if aabb.collides_aabb(b) {
                    VolumeQureyAABBResult::Mixed
                } else {
                    VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
                }
            }
            SlotMapCSGNodeData::Sphere(mat, v) => {
                let aabb = aabb.mul_mat(mat);

                let (min, max) = aabb.collides_unit_sphere();

                if max {
                    VolumeQureyAABBResult::Full(*v)
                } else if min {
                    VolumeQureyAABBResult::Mixed
                } else {
                    VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
                }
            }
            SlotMapCSGNodeData::All(v) => VolumeQureyAABBResult::Full(*v),
            SlotMapCSGNodeData::VoxelGrid(voxel_grid, ivec3) => todo!(),
        }
    }
}
