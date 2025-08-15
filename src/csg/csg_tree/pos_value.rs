use octa_force::glam::{vec3, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::aabb3d::AABB3, volume::VolumeQureyPosValue, voxel::palette::palette::MATERIAL_ID_NONE};

use super::tree::{CSGNodeData, CSGTree, CSGTreeKey};


impl VolumeQureyPosValue for CSGTree<u8> {
    fn get_value(&self, pos: Vec3A) -> u8 {
        self.get_pos_internal(pos, self.root_node)
    }
}

impl CSGTree<u8> {
    fn get_pos_internal(&self, pos: Vec3A, index: CSGTreeKey) -> u8 {
        let node = &self.nodes[index];

        if !node.aabb.pos_in_aabb(Vec4::from((pos, 1.0))) {
            return MATERIAL_ID_NONE;
        }

        match &node.data {
            CSGNodeData::Union(c1, c2) => {
                let a = self.get_pos_internal(pos, *c1);
                let b = self.get_pos_internal(pos, *c2);

                if a == b { a }
                else if a == MATERIAL_ID_NONE { b }
                else { a }
            }
            CSGNodeData::Remove(c1, c2) => {
                let a = self.get_pos_internal(pos, *c1);
                let b = self.get_pos_internal(pos, *c2);

                if b != MATERIAL_ID_NONE || a == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                else { a }
            }
            CSGNodeData::Intersect(c1, c2) => {
                let a = self.get_pos_internal(pos, *c1);
                let b = self.get_pos_internal(pos, *c2);

                if a == MATERIAL_ID_NONE || b == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                else { a }
            }
            CSGNodeData::Box(d) => d.get_value(pos), 
            CSGNodeData::Sphere(d) => d.get_value(pos), 
            CSGNodeData::All(d) => d.get_value(pos), 
            CSGNodeData::OffsetVoxelGrid(voxel_grid) => voxel_grid.get_value(pos),
            CSGNodeData::SharedVoxelGrid(shared_voxel_grid) => shared_voxel_grid.get_value(pos),
        }
    }
}
