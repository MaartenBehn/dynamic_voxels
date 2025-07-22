use core::fmt;

use octa_force::glam::{vec2, vec3, Vec2, Vec3A, Vec4};


use crate::{util::aabb2d::AABB2D, volume::VolumeQureyPosValid2D};

use super::tree::{CSGNodeData2D, CSGTree2D, CSGTreeKey2D};


impl<T: Default + Clone + fmt::Debug> VolumeQureyPosValid2D for CSGTree2D<T> {
    fn is_position_valid(&self, pos: Vec2) -> bool {
        self.is_pos_valid_internal(pos, self.root_node)
    }
}

impl<T: Default + Clone + fmt::Debug> CSGTree2D<T> {
    fn is_pos_valid_internal(&self, pos: Vec2, index: CSGTreeKey2D) -> bool {
        let node = &self.nodes[index];
        
        if !node.aabb.pos_in_aabb(pos) {
            return false;
        }

        match &node.data {
            CSGNodeData2D::Union(c1, c2) => {
                self.is_pos_valid_internal(pos, *c1) || self.is_pos_valid_internal(pos, *c2)
            }
            CSGNodeData2D::Remove(c1, c2) => {
                self.is_pos_valid_internal(pos, *c1) && !self.is_pos_valid_internal(pos, *c2)
            }
            CSGNodeData2D::Intersect(c1, c2) => {
                self.is_pos_valid_internal(pos, *c1) && self.is_pos_valid_internal(pos, *c2)
            }
            CSGNodeData2D::Box(mat, ..) => {
                let pos = mat.transform_point2(pos);

                let aabb = AABB2D::new(
                    vec2(-0.5, -0.5), 
                    vec2(0.5, 0.5));

                aabb.pos_in_aabb(pos)
            }
            CSGNodeData2D::Circle(mat, ..) => {
                let pos = mat.transform_point2(pos);

                pos.length_squared() < 1.0
            }
            CSGNodeData2D::All(..) => true,
        }
    }
} 
