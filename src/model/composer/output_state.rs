use octa_force::camera::Camera;

use crate::{mesh::scene::MeshSceneSend, scene::{dag_store::{SceneDAGKey}, worker::SceneWorkerSend}, voxel::{dag64::{VoxelDAG64, lod_heuristic::{LODHeuristicNone, LinearLODHeuristicSphere, PowHeuristicSphere}, parallel::ParallelVoxelDAG64}, palette::shared::SharedPalette}};


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

