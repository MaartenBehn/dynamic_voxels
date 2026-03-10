use std::time::Instant;

use octa_force::{OctaResult, camera::Camera, glam::{Mat4, Vec3A, vec3a}, log::info};

use crate::{VOXELS_PER_METER, scene::worker::SceneWorkerSend, util::default_types::{LODType, Volume}, voxel::{dag64::VoxelDAG64, palette::palette::MATERIAL_ID_BASE}};

#[derive(Debug)]
pub struct Editor {
    scene_send: SceneWorkerSend,
}

impl Editor {
    pub fn new(
        camera: &Camera,
        scene_send: SceneWorkerSend,
    ) -> OctaResult<Self> {
        
        let factor = 20.0;
        let mut csg = Volume::new_sphere_float(Vec3A::ZERO, 
            100.0 * factor, MATERIAL_ID_BASE);
        csg.cut_with_sphere(vec3a(70.0 * factor, 0.0, 0.0), 70.0 * factor, MATERIAL_ID_BASE);

        scene_send.add_dag_object(Mat4::IDENTITY, csg);

        Ok(Self {
            scene_send
        })
    }
}
