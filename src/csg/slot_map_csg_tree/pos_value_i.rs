use octa_force::glam::{vec3, IVec3, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::VolumeQureyPosValueI, voxel::renderer::palette::MATERIAL_ID_NONE};

use super::tree::{SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};


impl VolumeQureyPosValueI for SlotMapCSGTree<u8> {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        self.get_pos_internal_i(pos, self.root_node)
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        unreachable!()
    }
}

impl SlotMapCSGTree<u8> {
    fn get_pos_internal_i(&self, pos: IVec3, index: SlotMapCSGTreeKey) -> u8 {
        let node = &self.nodes[index];

        if !node.aabbi.pos_in_aabb(pos) {
            return MATERIAL_ID_NONE;
        }

        match &node.data {
            SlotMapCSGNodeData::Union(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if a == b { a }
                else if a == MATERIAL_ID_NONE { b }
                else { a }
            }
            SlotMapCSGNodeData::Remove(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if b != MATERIAL_ID_NONE || a == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                else { a }
            }
            SlotMapCSGNodeData::Intersect(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if a == MATERIAL_ID_NONE || b == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                else { a }
            }
            SlotMapCSGNodeData::Box(mat, v) => {
                let pos = mat.mul_vec4(Vec4::from((pos.as_vec3(), 1.0)));

                let aabb = AABB::new(
                    vec3(-0.5, -0.5, -0.5), 
                    vec3(0.5, 0.5, 0.5));

                if aabb.pos_in_aabb(pos) { *v }
                else { MATERIAL_ID_NONE }
            }
            SlotMapCSGNodeData::Sphere(mat, v) => {
                let pos = Vec3A::from(mat.mul_vec4(Vec4::from((pos.as_vec3(), 1.0))));

                if pos.length_squared() < 1.0 { *v }
                else { MATERIAL_ID_NONE }
            }
            SlotMapCSGNodeData::All(v) => *v,
            SlotMapCSGNodeData::OffsetVoxelGrid(voxel_grid) => voxel_grid.get_value_i(pos),
            SlotMapCSGNodeData::SharedVoxelGrid(shared_voxel_grid) => shared_voxel_grid.get_value_i(pos),
        }
    }
}
