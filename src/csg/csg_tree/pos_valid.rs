use core::fmt;

use octa_force::glam::{vec3, Vec3A, Vec4};

use crate::{util::aabb3d::AABB, volume::VolumeQureyPosValid};

use super::tree::{CSGNodeData, CSGTree, CSGTreeKey};

impl<T: Default + Clone + fmt::Debug> VolumeQureyPosValid for CSGTree<T> {
    fn is_position_valid_vec3(&self, pos: Vec4) -> bool {
        self.is_pos_valid_internal(pos, self.root_node)
    }
}

impl<T: Default + Clone + fmt::Debug> CSGTree<T> {
    fn is_pos_valid_internal(&self, pos: Vec4, index: CSGTreeKey) -> bool {
        let node = &self.nodes[index];

        if !node.aabb.pos_in_aabb(pos) {
            return false;
        }

        match &node.data {
            CSGNodeData::Union(c1, c2) => {
                        self.is_pos_valid_internal(pos, *c1) || self.is_pos_valid_internal(pos, *c2)
                    }
            CSGNodeData::Remove(c1, c2) => {
                        self.is_pos_valid_internal(pos, *c1) && !self.is_pos_valid_internal(pos, *c2)
                    }
            CSGNodeData::Intersect(c1, c2) => {
                        self.is_pos_valid_internal(pos, *c1) && self.is_pos_valid_internal(pos, *c2)
                    }
            CSGNodeData::Box(mat, ..) => {
                        let pos = mat.mul_vec4(pos);

                        let aabb = AABB::new(
                            vec3(-0.5, -0.5, -0.5), 
                            vec3(0.5, 0.5, 0.5));

                        aabb.pos_in_aabb(pos)
                    }
            CSGNodeData::Sphere(mat, ..) => {
                        let pos = Vec3A::from(mat.mul_vec4(pos));

                        pos.length_squared() < 1.0
                    }
            CSGNodeData::All(..) => true,
            CSGNodeData::OffsetVoxelGrid(voxel_grid) => todo!(),
            CSGNodeData::SharedVoxelGrid(shared_voxel_grid) => todo!(),
        }
    }
} 
