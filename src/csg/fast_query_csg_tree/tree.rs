use octa_force::glam::{ivec3, uvec3, vec3, vec4, EulerRot, IVec3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4, Vec4Swizzles};
use octa_force::log::{error, info};
use octa_force::puffin_egui::puffin;
use std::f32::consts::PI;
use std::{slice, usize};

use crate::util::aabb3d::AABB;
use crate::util::iaabb3d::AABBI;
use crate::volume::VolumeQureyAABBResult;
use crate::voxel::palette::palette::MATERIAL_ID_NONE;

pub const CSG_PARENT_NONE: usize = usize::MAX;

#[derive(Clone, Copy, Debug)]
pub enum FastQueryCSGNodeData<T> {
    Union(usize, usize),
    Remove(usize, usize),
    Intersect(usize, usize),
    Box(Mat4, T),
    Sphere(Mat4, T),
    All(T),
}

#[derive(Clone, Debug, Default)]
pub struct FastQueryCSGTree<T> {
    pub nodes: Vec<FastQueryCSGNodeData<T>>,
    pub aabb: AABB,
    pub aabbi: AABBI,
    pub root: usize,
}

impl<T: Copy> FastQueryCSGTree<T> {
    pub fn is_pos_valid_internal(&self, pos: Vec4, index: usize) -> bool {
        let node = self.nodes[index];

        match node {
            FastQueryCSGNodeData::Union(c1, c2) => {
                self.is_pos_valid_internal(pos, c1) || self.is_pos_valid_internal(pos, c2)
            }
            FastQueryCSGNodeData::Remove(c1, c2) => {
                self.is_pos_valid_internal(pos, c1) && !self.is_pos_valid_internal(pos, c2)
            }
            FastQueryCSGNodeData::Intersect(c1, c2) => {
                self.is_pos_valid_internal(pos, c1) && self.is_pos_valid_internal(pos, c2)
            }
            FastQueryCSGNodeData::Box(mat, ..) => {
                let pos = mat.mul_vec4(pos);

                let aabb = AABB::new(
                    vec3(-0.5, -0.5, -0.5), 
                    vec3(0.5, 0.5, 0.5));

                aabb.pos_in_aabb(pos)
            }
            FastQueryCSGNodeData::Sphere(mat, ..) => {
                let pos = Vec3A::from(mat.mul_vec4(pos));

                pos.length_squared() < 1.0
            }
            FastQueryCSGNodeData::All(..) => true,
        }
    }
}

impl FastQueryCSGTree<u8> {
    pub fn get_pos_internal(&self, pos: Vec4, index: usize) -> u8 {
        let node = self.nodes[index];

        match node { 
            FastQueryCSGNodeData::Union(c1, c2) => {
                let a = self.get_pos_internal(pos, c1);
                let b = self.get_pos_internal(pos, c2);

                if a == b { a }
                else if a == 0 { b }
                else { a }
            }
            FastQueryCSGNodeData::Remove(c1, c2) => {
                let a = self.get_pos_internal(pos, c1);
                let b = self.get_pos_internal(pos, c2);

                if b != 0 || a == 0 { 0 }
                else { a }
            }
            FastQueryCSGNodeData::Intersect(c1, c2) => {
                let a = self.get_pos_internal(pos, c1);
                let b = self.get_pos_internal(pos, c2);

                if a == 0 || b == 0 { 0 }
                else { a }
            }
            FastQueryCSGNodeData::Box(mat, v) => {
                let pos = mat.mul_vec4(pos);

                let aabb = AABB::new(
                    vec3(-0.5, -0.5, -0.5), 
                    vec3(0.5, 0.5, 0.5));

                if aabb.pos_in_aabb(pos) { v }
                else { 0 }
            }
            FastQueryCSGNodeData::Sphere(mat, v) => {
                let pos = Vec3A::from(mat.mul_vec4(pos));

                if pos.length_squared() < 1.0 { v }
                else { 0 }
            }
            FastQueryCSGNodeData::All(v) => v,
        }
    }

    pub fn get_pos_internal_i(&self, pos: IVec3, index: usize) -> u8 {
        let node = &self.nodes[index];

        match &node {
            FastQueryCSGNodeData::Union(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if a == b { a }
                else if a == MATERIAL_ID_NONE { b }
                else { a }
            }
            FastQueryCSGNodeData::Remove(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if b != MATERIAL_ID_NONE || a == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                else { a }
            }
            FastQueryCSGNodeData::Intersect(c1, c2) => {
                let a = self.get_pos_internal_i(pos, *c1);
                let b = self.get_pos_internal_i(pos, *c2);

                if a == MATERIAL_ID_NONE || b == MATERIAL_ID_NONE { MATERIAL_ID_NONE }
                else { a }
            }
            FastQueryCSGNodeData::Box(mat, v) => {
                let pos = mat.mul_vec4(Vec4::from((pos.as_vec3(), 1.0)));

                let aabb = AABB::new(
                    vec3(-0.5, -0.5, -0.5), 
                    vec3(0.5, 0.5, 0.5));

                if aabb.pos_in_aabb(pos) { *v }
                else { MATERIAL_ID_NONE }
            }
            FastQueryCSGNodeData::Sphere(mat, v) => {
                let pos = Vec3A::from(mat.mul_vec4(Vec4::from((pos.as_vec3(), 1.0))));

                if pos.length_squared() < 1.0 { *v }
                else { MATERIAL_ID_NONE }
            }
            FastQueryCSGNodeData::All(v) => *v,
        }
    }

    pub fn get_aabb_internal(&self, aabb: AABB, index: usize) -> VolumeQureyAABBResult  {
        let node = self.nodes[index];

        match node { 
            FastQueryCSGNodeData::Union(c1, c2) => {
                let a = self.get_aabb_internal(aabb, c1);
                let b = self.get_aabb_internal(aabb, c2);

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
            FastQueryCSGNodeData::Remove(c1, c2) => {
                let a = self.get_aabb_internal(aabb, c1);
                let b = self.get_aabb_internal(aabb, c2);

                if matches!(a, VolumeQureyAABBResult::Mixed) {
                    if matches!(b, VolumeQureyAABBResult::Mixed) {
                        return VolumeQureyAABBResult::Mixed;
                    } else if b.get_value() != 0 {
                        return VolumeQureyAABBResult::Full(0);
                    }
                }

                let a = a.get_value();
                if a == 0 {
                    return VolumeQureyAABBResult::Full(0);
                }

                if matches!(b, VolumeQureyAABBResult::Mixed) {
                    return VolumeQureyAABBResult::Mixed;
                }

                let b = b.get_value();
                if b != 0 { VolumeQureyAABBResult::Full(0) }
                else { VolumeQureyAABBResult::Full(a) }
            }
            FastQueryCSGNodeData::Intersect(c1, c2) => {
                let a = self.get_aabb_internal(aabb, c1);
                let b = self.get_aabb_internal(aabb, c2);
                
                if matches!(a, VolumeQureyAABBResult::Mixed) || matches!(b, VolumeQureyAABBResult::Mixed) {
                    return VolumeQureyAABBResult::Mixed;
                }

                let a = a.get_value();
                let b = b.get_value();

                if a == 0 || b == 0 { VolumeQureyAABBResult::Full(0) }
                else if a == b { VolumeQureyAABBResult::Full(a) }
                else { VolumeQureyAABBResult::Mixed }
            }
            FastQueryCSGNodeData::Box(mat, v) => {
                let aabb = aabb.mul_mat(&mat);

                let b = AABB::new(
                    vec3(-0.5, -0.5, -0.5), 
                    vec3(0.5, 0.5, 0.5));

                if aabb.contains_aabb(b) {
                    VolumeQureyAABBResult::Full(v)
                } else if aabb.collides_aabb(b) {
                    VolumeQureyAABBResult::Mixed
                } else {
                    VolumeQureyAABBResult::Full(0)
                }
            }
            FastQueryCSGNodeData::Sphere(mat, v) => {
                let aabb = aabb.mul_mat(&mat);

                let (min, max) = aabb.collides_unit_sphere();

                if max {
                    VolumeQureyAABBResult::Full(v)
                } else if min {
                    VolumeQureyAABBResult::Mixed
                } else {
                    VolumeQureyAABBResult::Full(0)
                }
            }
            FastQueryCSGNodeData::All(v) => VolumeQureyAABBResult::Full(v),
        }
    }

}






