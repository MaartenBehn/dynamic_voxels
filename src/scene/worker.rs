use core::fmt;
use std::{ops::Deref, sync::Arc};

use bvh::bvh::Bvh;
use octa_force::{OctaResult, glam::Mat4, log::{debug, trace, warn}, vulkan::{Buffer, Context, ash::vk, gpu_allocator::MemoryLocation}};
use parking_lot::Mutex;
use slotmap::{SlotMap, new_key_type};
use smol::{channel::Sender, future::FutureExt};

use crate::{mesh::Mesh, scene::{dag_store::SceneDAGStore, object::{SceneObject, SceneObjectData}, staging_copies::OptimalBufferCopyAlligment}, util::{buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, default_types::Volume, worker_response::{WithRespose, WorkerRespose}}, voxel::dag64::{DAG64Entry, VoxelDAG64, parallel::ParallelVoxelDAG64}};

use super::{dag64::{SceneAddDAGObject}, dag_store::SceneDAGKey, staging_copies::SceneStaging};

new_key_type! { pub struct SceneObjectKey; }

const INITAL_STAGING_BUFFER_AMMOUNT: usize = 2;
const INITAL_STAGING_BUFFER_SIZE: usize = 2;

const SCENE_TASK_QUEUE_SIZE: usize = 10;
const SCENE_STAGING_QUEUE_SIZE: usize = 2;

pub struct SceneWorker {
    pub staging_buffers: Vec<Buffer>,
    pub optimal_alignment: OptimalBufferCopyAlligment,

    pub objects: SlotMap<SceneObjectKey, SceneObject>,
    
    pub allocator: BuddyAllocator,
    pub bvh: Bvh<f32, 3>,
    pub bvh_allocation: ManualBuddyAllocation,
    pub bvh_len: usize,
    pub needs_bvh_update: bool,
    
    pub dag_store: SceneDAGStore,
}

#[derive(Debug)]
pub struct SceneWorkerRef {
    pub task: smol::Task<SceneWorker>,
    pub send: SceneWorkerSend,
    pub render_r: smol::channel::Receiver<SceneStaging>,
}

pub enum SceneTask {
    AddDAGObject(WithRespose<SceneAddDAGObject, SceneObjectKey>),
    RemoveObject(SceneObjectKey),
    FreeStagingBuffer(Buffer),
}

#[derive(Clone, Debug)]
pub struct SceneWorkerSend {
    s: smol::channel::Sender<SceneTask>,
}

impl SceneWorker {
    pub(super) fn new(buffer_size: usize, context: &Context) -> OctaResult<SceneWorker> {

        let mut staging_buffers = vec![];
        for _ in (0..INITAL_STAGING_BUFFER_AMMOUNT) {
            staging_buffers.push(context.create_buffer(
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
                MemoryLocation::CpuToGpu,
                buffer_size as _)?);
        }

        let mut allocator = BuddyAllocator::new(buffer_size, 32);
        let bvh = Bvh::build::<SceneObject>(&mut []);
        let bvh_allocation = allocator.alloc(1024)?;

        let mut dag_store = SceneDAGStore::new();

        let mut dag = VoxelDAG64::new(
            1000000, 
            1000000, 
        ).parallel();
        dag_store.add_dag(dag, &mut allocator).expect("Failed to add DAG to Store");

        let optimal_alignment = OptimalBufferCopyAlligment::new(context);

        Ok(SceneWorker {
            staging_buffers,
            optimal_alignment,

            allocator,
            objects: Default::default(),

            bvh,
            bvh_allocation,
            bvh_len: 0,
            needs_bvh_update: true,

            dag_store,
        })
    }

    pub(super) fn run(mut self) -> SceneWorkerRef {
        let (task_s, tasks_r) = smol::channel::bounded(SCENE_TASK_QUEUE_SIZE); 
        let (render_s, render_r) = smol::channel::bounded(SCENE_STAGING_QUEUE_SIZE); 

        let task = smol::spawn(async move {
            loop {
                match tasks_r.recv().await {
                    Ok(m) => {
                        debug!("Scene Worker Message: {m:?}");

                        match m {
                            SceneTask::FreeStagingBuffer(buffer) => {
                                self.staging_buffers.push(buffer);
                            },
                            SceneTask::AddDAGObject(worker_message) => {
                                let (data, awnser) = worker_message.unwarp();

                                let key = self.add_dag64_object(data)
                                    .expect("Failed to add DAG Object");

                                awnser(key);

                                self.update(&render_s).await.unwrap();
                            },
                            SceneTask::RemoveObject(key) => {
                                let res = self.remove_object(key);
                                if res.is_err() {
                                    warn!("Typed to removed Scene Object with invalid Key")
                                }

                                self.update(&render_s).await.unwrap();
                            },
                        }
                    },

                    Err(_) => {
                        break;
                    }, // Channel closed
                }
            } 
            self
        });

        SceneWorkerRef {
            task,
            send: SceneWorkerSend { s: task_s },
            render_r,
        }
    }

    async fn update(&mut self, render_s: &Sender<SceneStaging>) -> OctaResult<()> {
        if !self.needs_bvh_update && !self.dag_store.needs_update {
            return Ok(());
        }

        let mut builder = self.new_staging_builder();

        if self.dag_store.needs_update {
            self.dag_store.update(&mut builder); 
        }

        for object in self.objects.values_mut() {
            if object.needs_update {
                object.update(&self.dag_store, &mut builder);
            }
        }

        if self.needs_bvh_update {
            self.update_bvh(&mut builder)?;
        }

        render_s.send(builder.build()).await?;

        Ok(())
    }
}

impl SceneWorkerRef {
    pub fn stop(self) -> SceneWorker {
        self.send.s.close();
        smol::block_on(async {
            self.task.await
        })
    }
}

impl SceneWorkerSend {
    fn send(&self, message: SceneTask) {
        smol::block_on(async {
            self.s.send(message)
                .await.expect("Send channel to worker closed!");
        });
    }

    pub(super) fn free_staging_buffer(&self, buffer: Buffer) {
        self.send(SceneTask::FreeStagingBuffer(buffer));
    }

    pub fn add_dag_object(&self, mat: Mat4, model: Volume) -> WorkerRespose<SceneObjectKey> {
        let (message, res) = WithRespose::new(SceneAddDAGObject {
            mat,
            model,
        });

        self.send(SceneTask::AddDAGObject(message));
        res
    }

    pub fn remove_object(&self, object: SceneObjectKey) {
        self.send(SceneTask::RemoveObject(object));
    }
}

impl fmt::Debug for SceneTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddDAGObject(arg0) => f.debug_tuple("AddDAGObject").finish(),
            Self::RemoveObject(arg0) => f.debug_tuple("RemoveObject").finish(),
            Self::FreeStagingBuffer(arg0) => f.debug_tuple("FreeStagingBuffer").finish(),
        }
    }
}
