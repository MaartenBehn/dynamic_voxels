use octa_force::glam::{vec3, IVec3, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::VolumeQureyPosValueI, voxel::palette::palette::MATERIAL_ID_NONE};

use super::tree::{CSGNodeData, CSGTree, CSGTreeKey};


impl VolumeQureyPosValueI for CSGTree<u8> {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        self.get_pos_internal_i(pos, self.root_node)
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        unreachable!()
    }
}

impl CSGTree<u8> {
    fn get_pos_internal_i(&self, pos: IVec3, index: CSGTreeKey) -> u8 {
        let node = &self.nodes[index];

        if !node.aabbi.pos_in_aabb(pos) {
            return MATERIAL_ID_NONE;
        }

        match &node.data {
            CSGNodeData::Union(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if a == b { a }
                else if a == MATERIAL_ID_NONE { b }
                else { a }
            }
            CSGNodeData::Remove(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if b != MATERIAL_ID_NONE || a == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                else { a }
            }
            CSGNodeData::Intersect(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if a == MATERIAL_ID_NONE || b == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                else { a }
            }
            CSGNodeData::Box(d) => d.get_value_i(pos), 
            CSGNodeData::Sphere(d) => d.get_value_i(pos), 
            CSGNodeData::All(d) => d.get_value_i(pos), 
            CSGNodeData::OffsetVoxelGrid(d) => d.get_value_i(pos),
            CSGNodeData::SharedVoxelGrid(d) => d.get_value_i(pos),
        }
    }
}
