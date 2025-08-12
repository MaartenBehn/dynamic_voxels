use std::{ops::Deref, sync::Arc};

use octa_force::{log::{debug, warn}, vulkan::Context};
use parking_lot::Mutex;

use crate::{util::worker_message::{WithRespose, WorkerRespose}, voxel::dag64::parallel::ParallelVoxelDAG64};

use super::{dag64::{SceneAddDAGObject, SceneSetDAGEntry}, dag_store::SceneDAGKey, Scene, SceneData, SceneObjectKey};

pub enum SceneMessage {
    AddDAG(WithRespose<ParallelVoxelDAG64, SceneDAGKey>),
    AddDAGObject(WithRespose<SceneAddDAGObject, SceneObjectKey>),
    SetDAGEntry(SceneSetDAGEntry),
    Flush,
}

pub struct SceneWorker {
    task: smol::Task<Scene>,
    pub send: SceneWorkerSend,
    pub render_data: SceneWorkerRenderData,
}

#[derive(Clone, Debug)]
pub struct SceneWorkerSend {
    s: smol::channel::Sender<SceneMessage>,
}

#[derive(Clone)]
pub struct SceneWorkerData {
    data: Arc<Mutex<SceneData>>
}

#[derive(Clone, Debug)]
pub struct SceneWorkerRenderData {
    r: smol::channel::Receiver<SceneData>,
    data: SceneData,
}

impl Scene {
    pub fn run_worker(mut self, context: Context, cap: usize) -> SceneWorker {
        let (s, r) = smol::channel::bounded(cap); 
        let (render_s, render_r) = smol::channel::bounded(1); 
        let data = self.get_data();


        let task = smol::spawn(async move {
            loop {
                match r.recv().await {
                    Ok(m) => match m {
                        SceneMessage::AddDAG(worker_message) => {
                            let (data, awnser) = worker_message.unwarp();

                            let key = self.dag_store.add_dag(&context, data)
                                .expect("Failed to add DAG to Store");

                            awnser(key);
                        },
                        SceneMessage::AddDAGObject(worker_message) => {
                            let (data, awnser) = worker_message.unwarp();

                            let key = self.add_dag64_object(data)
                                .expect("Failed to add DAG Object");

                            awnser(key);
                        },
                        SceneMessage::SetDAGEntry(set) => {
                            let res = self.set_dag64_entry(set);
                            if res.is_err() {
                                warn!("Set Entry for invalid DAG Object -> Ignoring");
                            }
                        },
                        SceneMessage::Flush => {
                            self.flush();
                            let scene_data = self.get_data();
                            render_s.force_send(scene_data).expect("Failed to send Scene Data");
                        }
                    },

                    Err(_) => {
                        break;
                    }, // Channel closed
                }
            } 
            self
        });

        SceneWorker {
            task,
            send: SceneWorkerSend { s },
            render_data: SceneWorkerRenderData { r: render_r, data },
        }
    }
}

impl SceneWorker {
    pub fn stop(self) -> Scene {
        self.send.s.close();
        smol::block_on(async {
            self.task.await
        })
    }
}

impl SceneWorkerSend {
    fn send(&self, message: SceneMessage) {
        smol::block_on(async {
            self.s.send(message)
                .await.expect("Send channel to worker closed!");
        });
    }

    pub fn add_dag(&self, dag: ParallelVoxelDAG64) -> WorkerRespose<SceneDAGKey> {
        let (message, res) = WithRespose::new(dag);
        self.send(SceneMessage::AddDAG(message));
        res
    }

    pub fn add_dag_object(&self, dag: SceneAddDAGObject) -> WorkerRespose<SceneObjectKey> {
        let (message, res) = WithRespose::new(dag);
        self.send(SceneMessage::AddDAGObject(message));
        res
    }
}

impl SceneWorkerRenderData {
    pub fn get_data(&mut self) -> SceneData {
        match self.r.try_recv() {
            Ok(data) => {
                self.data = data;
                data
            },
            Err(e) => match e {
                smol::channel::TryRecvError::Empty => {
                    self.data
                },
                smol::channel::TryRecvError::Closed => {
                    panic!("Scene Worker Render Data Channel closed");
                },
            },
        }
    }
}
