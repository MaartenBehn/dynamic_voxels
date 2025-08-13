use octa_force::glam::{vec3, IVec3, Mat4, Quat, UVec3, Vec3, Vec3A, Vec4};

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::{VolumeBounds, VolumeBoundsI, VolumeQureyAABB, VolumeQureyAABBResult, VolumeQureyPosValid, VolumeQureyPosValue, VolumeQureyPosValueI}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{r#box::CSGBox, Base};

#[derive(Clone, Copy, Debug)]
pub struct CSGSphere<T> {
    mat: Mat4,
    v: T
}

impl<T: Base> CSGSphere<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        CSGSphere {
            mat: Mat4::from_scale_rotation_translation(
                Vec3::ONE * radius,
                Quat::IDENTITY,
                center,
            ).inverse(),
            v: T::base(),
        }
    }

    pub fn new_disk(center: Vec3, radius: f32, height: f32) -> Self {
        CSGSphere {
            mat: Mat4::from_scale_rotation_translation(
                vec3(radius, radius, height),
                Quat::IDENTITY,
                center,
            ).inverse(),
            v: T::base(),
        }
    }
}

impl<T> VolumeBounds for CSGSphere<T> {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB {
        let mat = self.mat.inverse();
        AABB::from_sphere(&mat)
    }
}

impl<T> VolumeBoundsI for CSGSphere<T> {
    fn calculate_bounds(&mut self) {}
    fn get_bounds_i(&self) -> AABBI { self.get_bounds().into() }
}

impl<T> VolumeQureyPosValid for CSGSphere<T> {
    fn is_position_valid_vec3(&self, pos: Vec3A) -> bool {
        let pos = Vec3A::from(self.mat.mul_vec4(Vec4::from((pos, 1.0))));
        pos.length_squared() < 1.0
    }
}

impl VolumeQureyPosValue for CSGSphere<u8> {
    fn get_value(&self, pos: Vec3A) -> u8 {
        if self.is_position_valid_vec3(pos) {
            self.v
        } else {
            MATERIAL_ID_NONE
        }
    }
}

impl VolumeQureyPosValueI for CSGSphere<u8> {
    fn get_value_i(&self, pos: IVec3) -> u8 {
        self.get_value(pos.as_vec3a())
    }

    fn get_value_relative_u(&self, pos: UVec3) -> u8 {
        unimplemented!()
    }
}

impl VolumeQureyAABB for CSGSphere<u8> {
    fn get_aabb_value(&self, aabb: AABB) -> VolumeQureyAABBResult {
        let aabb = aabb.mul_mat(&self.mat);

        let (min, max) = aabb.collides_unit_sphere();

        if max {
            VolumeQureyAABBResult::Full(self.v)
        } else if min {
            VolumeQureyAABBResult::Mixed
        } else {
            VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
        }
    }
}
