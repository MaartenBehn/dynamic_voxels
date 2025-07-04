use octa_force::glam::{ivec3, uvec3, vec3, vec4, EulerRot, IVec3, Mat4, Quat, UVec3, Vec3, Vec4Swizzles};
use octa_force::log::{error, info};
use octa_force::puffin_egui::puffin;
use std::f32::consts::PI;
use std::{slice, usize};

use crate::util::aabb::AABB;

pub const CSG_PARENT_NONE: usize = usize::MAX;

#[derive(Clone, Copy, Debug)]
pub enum FastPosQueryCSGNodeData {
    Union(usize, usize),
    Remove(usize, usize),
    Intersect(usize, usize),
    Box(Mat4),
    Sphere(Mat4),
    All,
}

#[derive(Clone, Debug, Default)]
pub struct FastPosQueryCSGTree {
    pub nodes: Vec<FastPosQueryCSGNodeData>,
    pub aabb: AABB,
}

impl FastPosQueryCSGTree {
    pub fn at_pos_internal(&self, pos: Vec3, index: usize) -> bool {
        let node = self.nodes[index];

        match node { 
            FastPosQueryCSGNodeData::Union(c1, c2) => {
                self.at_pos_internal(pos, c1) || self.at_pos_internal(pos, c2)
            }
            FastPosQueryCSGNodeData::Remove(c1, c2) => {
                self.at_pos_internal(pos, c1) && !self.at_pos_internal(pos, c2)
            }
            FastPosQueryCSGNodeData::Intersect(c1, c2) => {
                self.at_pos_internal(pos, c1) && self.at_pos_internal(pos, c2)
            }
            FastPosQueryCSGNodeData::Box(mat) => {
                let pos = mat.mul_vec4(vec4(pos.x, pos.y, pos.z, 1.0)).xyz();

                let aabb = AABB {
                    min: vec3(-0.5, -0.5, -0.5),
                    max: vec3(0.5, 0.5, 0.5),
                };

                aabb.pos_in_aabb(pos)
            }
            FastPosQueryCSGNodeData::Sphere(mat) => {
                let pos = mat.mul_vec4(vec4(pos.x, pos.y, pos.z, 1.0)).xyz();

                pos.length_squared() < 1.0
            }
            FastPosQueryCSGNodeData::All => true,
        }
    }
}






