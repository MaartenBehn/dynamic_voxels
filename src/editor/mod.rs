pub mod gizmo;

use std::time::Instant;

use octa_force::{OctaResult, camera::Camera, glam::{Mat4, Vec3A, vec3, vec3a}, log::info};

use crate::{VOXELS_PER_METER, scene::worker::{SceneObjectKey, SceneWorkerSend}, util::{default_types::{LODType, Volume}, worker_response::WorkerRespose}, voxel::{grid::offset, palette::palette::MATERIAL_ID_BASE}};

#[derive(Debug)]
pub struct Editor {
    scene_send: SceneWorkerSend,
    key: SceneObjectKey,
    model: Volume,
    index: usize,
    mat: Mat4,
}

impl Editor {
    pub fn new(
        camera: &Camera,
        scene_send: SceneWorkerSend,
    ) -> OctaResult<Self> {
        
        let factor = 4.0;
        let mut model = Volume::new_sphere_float(Vec3A::new(0.0, 0.0, 0.0), 
            100.0 * factor, MATERIAL_ID_BASE);
        let res = model.cut_with_sphere(
            vec3a(80.0 * factor, 0.0, 0.0), 
            30.0 * factor, 
            MATERIAL_ID_BASE);

        let mat = model.get_mat(res.new_object_index);

        let key = scene_send.add_object(Mat4::from_rotation_x(0.0_f32.to_radians()), model.clone()).result_blocking();
        
        //scene_send.debug_probes(key, true);

        Ok(Self {
            scene_send,
            key,
            model,
            index: res.new_object_index,
            mat,
        })
    }

    pub fn update(&mut self, time: f32) {

        /*
        if let Some(res) = self.respose.as_ref() {
            if !res.has_result() {
                return;
            }
        }

        let interval = 10.0;
        let size = 200.0;
        let t = ((time % interval) / 10.0);
        let t = simple_easing::cubic_in_out(simple_easing::roundtrip(t));

        let new_mat = self.mat.mul_mat4(&Mat4::from_translation(vec3(t * size, 0.0, 0.0)));
        
        self.model.reset_changed_bounds();
        self.model.set_mat(self.index, new_mat);

        self.respose = Some(self.scene_send.update_model(self.key, self.model.clone()));
        */    
    }
}
