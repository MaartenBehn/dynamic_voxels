use octa_force::camera::Camera;

use crate::{mesh::scene::MeshSceneSend, scene::worker::SceneWorkerSend, voxel::palette::shared::SharedPalette};

#[derive(Debug, Clone)]
pub struct OutputState {
    pub scene: SceneWorkerSend,
    pub mesh_scene: MeshSceneSend,
}

impl OutputState {
    pub fn new(scene: SceneWorkerSend, mesh_scene: MeshSceneSend, camera: &Camera, palette: SharedPalette) -> Self {
        Self {
            scene,
            mesh_scene,
        } 
    }
}

