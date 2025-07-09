use octa_force::glam::{vec3, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::aabb::AABB, volume::VolumeQureyPosValue, voxel::renderer::palette::MATERIAL_ID_NONE};

use super::tree::{SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};


impl VolumeQureyPosValue for SlotMapCSGTree<u8> {
    fn get_value(&self, pos: Vec3) -> u8 {
        self.get_pos_internal(Vec4::from((pos, 1.0)), self.root_node)
    }

    fn get_value_a(&self, pos: Vec3A) -> u8 {
        self.get_pos_internal(Vec4::from((pos, 1.0)), self.root_node)
    }

    fn get_value_u(&self, pos: UVec3) -> u8 {
        self.get_value(pos.as_vec3())
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
                else if a == 0 { b }
                else { a }
            }
            SlotMapCSGNodeData::Remove(c1, c2) => {
                let a = self.get_pos_internal(pos, *c1);
                let b = self.get_pos_internal(pos, *c2);

                if b != 0 || a == 0 { 0 }
                else { a }
            }
            SlotMapCSGNodeData::Intersect(c1, c2) => {
                let a = self.get_pos_internal(pos, *c1);
                let b = self.get_pos_internal(pos, *c2);

                if a == 0 || b == 0 { 0 }
                else { a }
            }
            SlotMapCSGNodeData::Mat(mat, c) => {
                let pos = mat.mul_vec4(pos);
                self.get_pos_internal(pos, *c)
            },
            SlotMapCSGNodeData::Box(mat, v) => {
                let pos = mat.mul_vec4(pos);

                let aabb = AABB::new(
                    vec3(-0.5, -0.5, -0.5), 
                    vec3(0.5, 0.5, 0.5));

                if aabb.pos_in_aabb(pos) { *v }
                else { 0 }
            }
            SlotMapCSGNodeData::Sphere(mat, v) => {
                let pos = Vec3A::from(mat.mul_vec4(pos));

                if pos.length_squared() < 1.0 { *v }
                else { 0 }
            }
            SlotMapCSGNodeData::All(v) => *v,
            SlotMapCSGNodeData::VoxelGrid(voxel_grid, ivec3) => todo!(),
        }
    }
}
