use octa_force::glam::{IVec3, Mat4, Quat, Vec3};
use slotmap::new_key_type;

use crate::{util::shader_constants::VOXELS_PER_SHADER_UNIT, voxel::dag64::util::get_voxel_size};

new_key_type! { pub struct DAG64EntryKey; }

#[derive(Debug, Clone, Copy)]
pub struct DAG64Entry {
    pub levels: u8,
    pub root_index: u32,
    pub offset: IVec3,
}

impl DAG64Entry { 
    pub fn get_size(&self) -> u32 {
        get_voxel_size(self.levels) as u32
    }

    pub fn calc_mat(&self, mat: Mat4) -> Mat4 {

        let size = self.get_size() as f32;
        let scale = (VOXELS_PER_SHADER_UNIT as f32 / size);

        let mat = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale), 
            Quat::IDENTITY,
            Vec3::splat(1.0) - self.offset.as_vec3() / size,
        ).mul_mat4(&mat.inverse())
            .transpose();

        mat
    }
}
