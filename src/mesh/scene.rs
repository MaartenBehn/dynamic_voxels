use octa_force::{camera::Camera, engine::Engine, glam::Mat4, vulkan::{CommandBuffer, Context, Swapchain, ash::vk::AttachmentLoadOp}};
use slotmap::{SlotMap, new_key_type};
use smol::channel::Receiver;

use crate::{mesh::{Mesh, gpu_mesh::GPUMesh, renderer::MeshRenderer}, util::worker_response::{WithRespose, WorkerRespose}};

new_key_type! { pub struct SceneMeshKey; }

#[derive(Debug)]
pub struct MeshScene {
    pub meshes: SlotMap<SceneMeshKey, MeshObject>, 
    pub send: MeshSceneSend,
    r: Receiver<MeshSceneMessage>,
    renderer: MeshRenderer,
}

#[derive(Debug)]
pub struct MeshObject {
    mesh: GPUMesh,
    mat: Mat4,
}

#[derive(Debug)]
pub enum MeshSceneMessage {
    Add(WithRespose<MeshSceneMessageAdd, SceneMeshKey>),
    Remove(SceneMeshKey)
}

#[derive(Debug)]
pub struct MeshSceneMessageAdd {
    mat: Mat4,
    mesh: Mesh,
}

#[derive(Clone, Debug)]
pub struct MeshSceneSend {
    s: smol::channel::Sender<MeshSceneMessage>,
}

impl MeshScene {
    pub fn new(context: &Context, swapchain: &Swapchain) -> Self {
        let (mesh_s, mesh_r) = smol::channel::bounded(1);

        let renderer = MeshRenderer::new(context, swapchain.format, swapchain.depth_format);
        
        Self {
            meshes: Default::default(),
            send: MeshSceneSend { s: mesh_s },
            r: mesh_r,
            renderer,
        }
    }

    pub fn update(&mut self, context: &Context) {
        while let Ok(message) = self.r.try_recv() {
            match message {
                MeshSceneMessage::Add(add) => {
                    let (data, awnser) = add.unwarp();
                    let mesh = data.mesh.flush_to_gpu(context);
                    let key = self.meshes.insert(MeshObject { mesh, mat: data.mat });
                    
                    awnser(key);
                },
                MeshSceneMessage::Remove(scene_mesh_key) => {
                    self.meshes.remove(scene_mesh_key);
                },
            }
        }
    }

    pub fn render(&self, buffer: &CommandBuffer, camera: &Camera, engine: &Engine) {

        self.renderer.start_render(buffer, engine, camera);
         
        for mesh in self.meshes.values() {
            self.renderer.render(buffer, &mesh.mesh);
        }

        buffer.end_rendering();
    }
}

impl MeshSceneSend {
    fn send(&self, message: MeshSceneMessage) {
        smol::block_on(async {
            self.s.send(message)
                .await.expect("Send channel to mesh closed!");
        });
    }

    pub fn add(&self, mesh: Mesh, mat: Mat4) -> WorkerRespose<SceneMeshKey> {
        let (message, res) = WithRespose::new(MeshSceneMessageAdd {
            mesh, 
            mat
        });

        self.send(MeshSceneMessage::Add(message));
        res
    }

    
    pub fn remove(&self, key: SceneMeshKey) {
        self.send(MeshSceneMessage::Remove(key));
    }
}

