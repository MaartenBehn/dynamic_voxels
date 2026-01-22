use octa_force::camera::Camera;

use crate::{mesh::scene::MeshSceneSend, scene::{dag_store::SceneDAGKey, worker::SceneWorkerSend}, voxel::{dag64::{VoxelDAG64, parallel::ParallelVoxelDAG64}, palette::shared::SharedPalette}};


#[derive(Debug, Clone)]
pub struct OutputState {
    pub dag: ParallelVoxelDAG64,
    pub scene: SceneWorkerSend,
    pub scene_dag_key: SceneDAGKey,
    pub mesh_scene: MeshSceneSend,
}

impl OutputState {
    pub fn new(scene: SceneWorkerSend, mesh_scene: MeshSceneSend, camera: &Camera, palette: SharedPalette) -> Self {
        let mut dag = VoxelDAG64::new(1000000, 1000000).parallel();
        let scene_dag_key = scene.add_dag(dag.clone()).result_blocking();

        Self {
            dag,
            scene,
            scene_dag_key,
            mesh_scene,
        } 
    }
}

