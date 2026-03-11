use core::fmt;
use std::{ops::Deref, sync::Arc};

use octa_force::{OctaResult, anyhow::bail, camera::Camera, glam::{Mat4, Vec3, Vec3A}, log::{debug, error, trace, warn}, vulkan::{Buffer, Context, ash::vk, gpu_allocator::MemoryLocation}};
use parking_lot::Mutex;
use slotmap::{SlotMap, new_key_type};
use smol::{channel::Sender, future::FutureExt};

use crate::{VOXELS_PER_METER, bvh::Bvh, mesh::Mesh, scene::{dag_store::SceneDAGStore, object::{BVHExtraData, BVHObject, SceneObject, SceneObjectData}, staging_copies::OptimalBufferCopyAlligment}, util::{buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, default_types::{LODType, Volume}, worker_response::{WithRespose, WorkerRespose}}, voxel::dag64::{DAG64Entry, VoxelDAG64, lod_heuristic::LODHeuristicT, parallel::ParallelVoxelDAG64}};

use super::{dag64::{SceneAddDAGObject}, dag_store::SceneDAGKey, staging_copies::SceneStaging};

new_key_type! { pub struct SceneObjectKey; }

const INITAL_STAGING_BUFFER_AMMOUNT: usize = 5;
const INITAL_STAGING_BUFFER_SIZE: usize = 2;

const SCENE_TASK_QUEUE_SIZE: usize = 10;
const SCENE_STAGING_QUEUE_SIZE: usize = 2;

pub struct SceneWorker {
    pub staging_buffers: Vec<Buffer>,
    pub optimal_alignment: OptimalBufferCopyAlligment,

    pub objects: SlotMap<SceneObjectKey, SceneObject>,
    
    pub allocator: BuddyAllocator,
    pub bvh: Bvh<SceneObjectData, BVHExtraData, Vec3, f32, 3>,
    pub bvh_allocation: ManualBuddyAllocation,
    pub bvh_len: usize,
    pub needs_bvh_update: bool,
    
    pub dag_store: SceneDAGStore,
    pub lod: LODType,
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
    GetObjectMat(WithRespose<SceneObjectKey, Mat4>),
    UpdateObjectMat((SceneObjectKey, Mat4)),
    UpdateModel(WithRespose<(SceneObjectKey, Volume), ()>),

    FreeStagingBuffer(Buffer),
    CameraPosition(Vec3),
}

#[derive(Clone, Debug)]
pub struct SceneWorkerSend {
    task_s: smol::channel::Sender<SceneTask>,
    free_staging_buffer_s: smol::channel::Sender<SceneTask>,
    cam_pos_s: smol::channel::Sender<SceneTask>,
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
        let optimal_alignment = OptimalBufferCopyAlligment::new(context);

        let mut allocator = BuddyAllocator::new(buffer_size, 32);
        let bvh = Bvh::empty();
        let bvh_allocation = allocator.alloc(1024)?;

        let mut dag_store = SceneDAGStore::new();

        let mut dag = ParallelVoxelDAG64::new(
            20000000, 
            4000000, 
        );
        dag_store.add_dag(dag, &mut allocator).expect("Failed to add DAG to Store");

        let lod = LODType::default();

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
            lod,
        })
    }

    pub(super) fn run(mut self) -> SceneWorkerRef {
        let (task_s, tasks_r) = smol::channel::bounded(SCENE_TASK_QUEUE_SIZE); 
        let (free_staging_buffer_s, free_staging_buffer_r) 
            = smol::channel::unbounded(); 
        let (cam_pos_s, cam_pos_r) = smol::channel::bounded(1); 
        let (render_s, render_r) = smol::channel::bounded(SCENE_STAGING_QUEUE_SIZE); 

        let task = smol::spawn(async move {
            loop {
                match free_staging_buffer_r.recv().or(cam_pos_r.recv()).or(tasks_r.recv()).await {
                    Ok(m) => {
                        debug!("Scene Worker Message: {m:?}");

                        match m {
                            SceneTask::FreeStagingBuffer(buffer) => {
                                self.staging_buffers.push(buffer);
                            },
                            SceneTask::CameraPosition(pos) => {
                                self.lod.set_center((pos * VOXELS_PER_METER as f32).as_ivec3());
    
                                self.rebuild_all_dag_objects();
                                self.update(&render_s).await.unwrap();
                                self.clean();
                            }
                            SceneTask::AddDAGObject(worker_message) => {
                                let (data, awnser) = worker_message.unwarp();

                                let key = self.add_dag64_object(data)
                                    .expect("Failed to add DAG Object");

                                awnser(key);

                                self.update(&render_s).await.unwrap();
                                self.clean();
                            },
                            SceneTask::RemoveObject(key) => {
                                let res = self.remove_object(key);
                                if res.is_err() {
                                    warn!("Typed to removed Scene Object with invalid Key")
                                }

                                self.update(&render_s).await.unwrap();
                            },
                            SceneTask::GetObjectMat(worker_message) => {
                                let (key, awnser) = worker_message.unwarp();
                                
                                if let Some(o) = self.objects.get(key) {
                                    awnser(o.get_mat());
                                } else {
                                    error!("GetObjectMat: Invalid key");
                                }
                            },
                            SceneTask::UpdateObjectMat((key, mat)) => {
                                if let Some(o) = self.objects.get_mut(key) {
                                    o.update_mat(mat);
                                } else {
                                    error!("UpdateObjectMat: Invalid key");
                                }

                                self.needs_bvh_update = true;
                                self.update(&render_s).await.unwrap();
                            },
                            SceneTask::UpdateModel(worker_message) => {
                                let ((key, model), awnser) = worker_message.unwarp();
                                
                                if let Some(o) = self.objects.get_mut(key) {
                                    let dag_o = o.get_dag_object_mut();
                                    dag_o.model = model;
                                    dag_o.rebuild_changed(&mut self.dag_store, &self.lod);
                                    o.needs_update = true;
                                } else {
                                    error!("UpdateModel: Invalid key");
                                }

                                awnser(());

                                self.needs_bvh_update = true;
                                self.update(&render_s).await.unwrap();
                                self.clean();
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
            send: SceneWorkerSend { 
                task_s,
                free_staging_buffer_s,
                cam_pos_s,
            },
            render_r,
        }
    }

    async fn update(&mut self, render_s: &Sender<SceneStaging>) -> OctaResult<()> {
        if !self.needs_bvh_update && !self.dag_store.needs_update {
            return Ok(());
        }

        let mut builder = self.new_staging_builder();

        self.dag_store.update(&mut builder); 

        for object in self.objects.values_mut() {
            object.update(&self.dag_store, &mut builder);
        }

        self.update_bvh(&mut builder)?;

        #[cfg(debug_assertions)]
        debug!("Scene Worker: Sending Staging Buffer");

        render_s.send(builder.build()).await?;

        Ok(())
    }

    fn clean(&mut self) {
        self.dag_store.clean(&mut self.objects);
    }
}

impl SceneWorkerRef {
    pub fn stop(self) -> SceneWorker {
        self.send.task_s.close();
        self.send.free_staging_buffer_s.close();
        self.send.cam_pos_s.close();
        smol::block_on(async {
            self.task.await
        })
    }
}

impl SceneWorkerSend {
    
    pub(super) fn free_staging_buffer(&self, buffer: Buffer) {
        smol::block_on(async {
            self.free_staging_buffer_s.send(SceneTask::FreeStagingBuffer(buffer))
                .await.expect("Send channel to worker closed!");
        });
    }

    pub(super) fn camera_position(&self, pos: Vec3) {
        self.cam_pos_s.force_send(SceneTask::CameraPosition(pos))
            .expect("Send channel to worker closed!");
    }

    fn send_task(&self, message: SceneTask) {
        smol::block_on(async {
            self.task_s.send(message)
                .await.expect("Send channel to worker closed!");
        });
    }

    pub fn add_dag_object(&self, mat: Mat4, model: Volume) -> WorkerRespose<SceneObjectKey> {
        let (message, res) = WithRespose::new(SceneAddDAGObject {
            mat,
            model,
        });

        self.send_task(SceneTask::AddDAGObject(message));
        res
    }

    pub fn remove_object(&self, object: SceneObjectKey) {
        self.send_task(SceneTask::RemoveObject(object));
    }

    pub fn get_object_mat(&self, object: SceneObjectKey) -> WorkerRespose<Mat4> {
        let (message, res) = WithRespose::new(object);
        self.send_task(SceneTask::GetObjectMat(message));
        res
    }  

    pub fn update_object_mat(&self, object: SceneObjectKey, mat: Mat4) {
        self.send_task(SceneTask::UpdateObjectMat((object, mat)));
    } 
    
    pub fn update_model(&self, object: SceneObjectKey, model: Volume) ->  WorkerRespose<()> {
        let (message, res) = WithRespose::new((object, model));
        self.send_task(SceneTask::UpdateModel(message));
        res
    }   
}

impl fmt::Debug for SceneTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddDAGObject(arg0) => f.debug_tuple("AddDAGObject").finish(),
            Self::RemoveObject(arg0) => f.debug_tuple("RemoveObject").finish(),
            Self::FreeStagingBuffer(arg0) => f.debug_tuple("FreeStagingBuffer").finish(),
            Self::CameraPosition(arg0) => f.debug_tuple("CameraPosition").finish(),
            Self::GetObjectMat(arg0) => f.debug_tuple("GetObjectMat").finish(),
            Self::UpdateObjectMat(arg0) => f.debug_tuple("UpdateObjectMat").finish(),
            Self::UpdateModel(arg0) => f.debug_tuple("UpdateModel").finish(),
        }
    }
}
