use octa_force::glam::{vec3, IVec3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::{VolumeBounds, VolumeBoundsI, VolumeQureyAABB, VolumeQureyAABBI, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue, VolumeQureyPosValueI}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::Base;

#[derive(Clone, Copy, Debug)]
pub struct CSGBox<T> {
    mat: Mat4,
    v: T
}

impl<T: Base> CSGBox<T> {
    pub fn new(mat: Mat4) -> Self {
        CSGBox {
            mat: mat.inverse(),
            v: T::base(),
        }
    }
}

impl<T> VolumeBounds for CSGBox<T> {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB {
        let mat = self.mat.inverse();
        AABB::from_box(&mat)
    }
}

impl<T> VolumeBoundsI for CSGBox<T> {
    fn calculate_bounds_i(&mut self) {}
    fn get_bounds_i(&self) -> AABBI { self.get_bounds().into() }
}

impl<T> VolumeQureyPosValid for CSGBox<T> {
    fn is_position_valid_vec3(&self, pos: Vec3A) -> bool {
        let pos = self.mat.mul_vec4(Vec4::from((pos, 1.0)));
        let aabb = AABB::new(
            vec3(-0.5, -0.5, -0.5), 
            vec3(0.5, 0.5, 0.5));

        aabb.pos_in_aabb(pos)
    }
}

impl VolumeQureyPosValue for CSGBox<u8> {
    fn get_value(&self, pos: Vec3A) -> u8 {
        if self.is_position_valid_vec3(pos) {
            self.v
        } else {
            MATERIAL_ID_NONE
        }
    }
}

impl VolumeQureyPosValueI for CSGBox<u8> {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        self.get_value(pos.as_vec3a())
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        unimplemented!()
    }
}

impl VolumeQureyAABB for CSGBox<u8> {
    fn get_aabb_value(&self, aabb: AABB) -> VolumeQureyAABBResult {
        let aabb = aabb.mul_mat(&self.mat);

        let b = AABB::new(
            vec3(-0.5, -0.5, -0.5), 
            vec3(0.5, 0.5, 0.5));

        if aabb.contains_aabb(b) {
            VolumeQureyAABBResult::Full(self.v)
        } else if aabb.collides_aabb(b) {
            VolumeQureyAABBResult::Mixed
        } else {
            VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
        }

    }
}

impl VolumeQureyAABBI for CSGBox<u8> {
    fn get_aabb_value_i(&self, aabb: AABBI) -> VolumeQureyAABBResult {
        self.get_aabb_value(aabb.into())
    }
}
