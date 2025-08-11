use crate::{util::worker_message::WorkerMessage, voxel::dag64::parallel::ParallelVoxelDAG64};

use super::Scene;

pub enum SceneMessages {
    ADD_DAG(WorkerMessage<ParallelVoxelDAG64>),
    ADD_DAG_OBJECT(WorkerMessage<>)
}

pub struct SceneWorker {
    scene: Scene,

}
