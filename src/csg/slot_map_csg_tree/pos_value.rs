use octa_force::glam::{vec3, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::aabb3d::AABB, volume::VolumeQureyPosValue, voxel::renderer::palette::MATERIAL_ID_NONE};

use super::tree::{SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};


impl VolumeQureyPosValue for SlotMapCSGTree<u8> {
    fn get_value(&self, pos: Vec3A) -> u8 {
        self.get_pos_internal(Vec4::from((pos, 1.0)), self.root_node)
    }
}

impl SlotMapCSGTree<u8> {
    fn get_pos_internal(&self, pos: Vec4, index: SlotMapCSGTreeKey) -> u8 {
        let node = &self.nodes[index];

        if !node.aabb.pos_in_aabb(pos) {
            return MATERIAL_ID_NONE;
        }

        match &node.data {
            SlotMapCSGNodeData::Union(c1, c2) => {
                        let a = self.get_pos_internal(pos, *c1);
                        let b = self.get_pos_internal(pos, *c2);

                        if a == b { a }
                        else if a == MATERIAL_ID_NONE { b }
                        else { a }
                    }
            SlotMapCSGNodeData::Remove(c1, c2) => {
                        let a = self.get_pos_internal(pos, *c1);
                        let b = self.get_pos_internal(pos, *c2);

                        if b != MATERIAL_ID_NONE || a == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                        else { a }
                    }
            SlotMapCSGNodeData::Intersect(c1, c2) => {
                        let a = self.get_pos_internal(pos, *c1);
                        let b = self.get_pos_internal(pos, *c2);

                        if a == MATERIAL_ID_NONE || b == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                        else { a }
                    }
            SlotMapCSGNodeData::Box(mat, v) => {
                        let pos = mat.mul_vec4(pos);

                        let aabb = AABB::new(
                            vec3(-0.5, -0.5, -0.5), 
                            vec3(0.5, 0.5, 0.5));

                        if aabb.pos_in_aabb(pos) { *v }
                        else { MATERIAL_ID_NONE }
                    }
            SlotMapCSGNodeData::Sphere(mat, v) => {
                        let pos = Vec3A::from(mat.mul_vec4(pos));

                        if pos.length_squared() < 1.0 { *v }
                        else { MATERIAL_ID_NONE }
                    }
            SlotMapCSGNodeData::All(v) => *v,
            SlotMapCSGNodeData::OffsetVoxelGrid(voxel_grid) => voxel_grid.get_value(Vec3A::from(pos)),
            SlotMapCSGNodeData::SharedVoxelGrid(shared_voxel_grid) => shared_voxel_grid.get_value(Vec3A::from(pos)),
        }
    }
}
