use core::fmt;

use octa_force::glam::{vec3, Vec3A, Vec4};

use crate::{util::aabb3d::AABB, volume::VolumeQureyPosValid};

use super::tree::{SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};

impl<T: Default + Clone + fmt::Debug> VolumeQureyPosValid for SlotMapCSGTree<T> {
    fn is_position_valid_vec3(&self, pos: Vec4) -> bool {
        self.is_pos_valid_internal(pos, self.root_node)
    }
}

impl<T: Default + Clone + fmt::Debug> SlotMapCSGTree<T> {
    fn is_pos_valid_internal(&self, pos: Vec4, index: SlotMapCSGTreeKey) -> bool {
        let node = &self.nodes[index];

        if !node.aabb.pos_in_aabb(pos) {
            return false;
        }

        match &node.data {
            SlotMapCSGNodeData::Union(c1, c2) => {
                        self.is_pos_valid_internal(pos, *c1) || self.is_pos_valid_internal(pos, *c2)
                    }
            SlotMapCSGNodeData::Remove(c1, c2) => {
                        self.is_pos_valid_internal(pos, *c1) && !self.is_pos_valid_internal(pos, *c2)
                    }
            SlotMapCSGNodeData::Intersect(c1, c2) => {
                        self.is_pos_valid_internal(pos, *c1) && self.is_pos_valid_internal(pos, *c2)
                    }
            SlotMapCSGNodeData::Box(mat, ..) => {
                        let pos = mat.mul_vec4(pos);

                        let aabb = AABB::new(
                            vec3(-0.5, -0.5, -0.5), 
                            vec3(0.5, 0.5, 0.5));

                        aabb.pos_in_aabb(pos)
                    }
            SlotMapCSGNodeData::Sphere(mat, ..) => {
                        let pos = Vec3A::from(mat.mul_vec4(pos));

                        pos.length_squared() < 1.0
                    }
            SlotMapCSGNodeData::All(..) => true,
            SlotMapCSGNodeData::OffsetVoxelGrid(voxel_grid) => todo!(),
            SlotMapCSGNodeData::SharedVoxelGrid(shared_voxel_grid) => todo!(),
        }
    }
} 
