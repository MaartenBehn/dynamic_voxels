use core::fmt;
use std::{ops::Deref, sync::Arc};

use octa_force::{glam::Mat4, log::{debug, trace, warn}, vulkan::{AllocContext, Context}};
use parking_lot::Mutex;
use smol::future::FutureExt;

use crate::{util::worker_response::{WithRespose, WorkerRespose}, voxel::dag64::{parallel::ParallelVoxelDAG64, DAG64Entry}};

use super::{dag64::{SceneAddDAGObject, SceneSetDAGEntry}, dag_store::SceneDAGKey, Scene, SceneData, SceneObjectKey};

pub enum SceneMessage {
    AddDAG(WithRespose<ParallelVoxelDAG64, SceneDAGKey>),
    AddDAGObject(WithRespose<SceneAddDAGObject, SceneObjectKey>),
    SetDAGEntry(SceneSetDAGEntry),
    RemoveObject(SceneObjectKey),
    Flush,
}

pub struct SceneWorker {
    pub task: smol::Task<Scene>,
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
    pub fn run_worker(mut self, context: AllocContext, cap: usize) -> SceneWorker {
        let (s, r) = smol::channel::bounded(cap); 
        let (flush_s, flush_r) = smol::channel::bounded(1); 
        let (render_s, render_r) = smol::channel::bounded(1); 
        let data = self.get_data();

        let task = smol::spawn(async move {
            loop {
                match r.recv().or(flush_r.recv()).await {
                    Ok(m) => {
                        debug!("Scene Worker Message: {m:?}");

                        match m {
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
                                let _ = flush_s.force_send(SceneMessage::Flush);
                            },
                            SceneMessage::SetDAGEntry(set) => {
                                let res = self.set_dag64_entry(set);
                                if res.is_err() {
                                    warn!("Set Entry for invalid DAG Object -> Ignoring");
                                }
                                let _ = flush_s.force_send(SceneMessage::Flush);
                            },
                            SceneMessage::RemoveObject(key) => {
                                let res = self.remove_object(key);
                                if res.is_err() {
                                    warn!("Typed to removed Scene Object with invalid Key")
                                }

                                let _ = flush_s.force_send(SceneMessage::Flush);
                            },
                            SceneMessage::Flush => {
                                self.flush().expect("Failed to flush scene");
                                let scene_data = self.get_data();
                                render_s.force_send(scene_data).expect("Failed to send Scene Data");
                            }
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

    pub fn add_dag_object(&self, mat: Mat4, dag_key: SceneDAGKey, entry: DAG64Entry) -> WorkerRespose<SceneObjectKey> {
        let (message, res) = WithRespose::new(SceneAddDAGObject {
            mat,
            dag_key,
            entry,
        });

        self.send(SceneMessage::AddDAGObject(message));
        res
    }

    pub fn set_dag_entry(&self, object: SceneObjectKey, entry: DAG64Entry) {
        self.send(SceneMessage::SetDAGEntry(SceneSetDAGEntry {
            object,
            entry,
        }));
    }

    pub fn remove_object(&self, object: SceneObjectKey) {
        self.send(SceneMessage::RemoveObject(object));
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

impl fmt::Debug for SceneWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SceneWorker")
            .field("task", &())
            .field("send", &self.send)
            .field("render_data", &self.render_data)
            .finish()
    }
}

impl fmt::Debug for SceneMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddDAG(arg0) => f.debug_tuple("AddDAG").finish(),
            Self::AddDAGObject(arg0) => f.debug_tuple("AddDAGObject").finish(),
            Self::SetDAGEntry(arg0) => f.debug_tuple("SetDAGEntry").finish(),
            Self::RemoveObject(arg0) => f.debug_tuple("RemoveObject").finish(),
            Self::Flush => write!(f, "Flush"),
        }
    }
}
