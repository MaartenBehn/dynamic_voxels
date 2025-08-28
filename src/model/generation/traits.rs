use octa_force::{glam::{Vec2, Vec3A}, OctaResult};

use crate::{model::worker::ModelChangeSender, scene::worker::SceneWorkerSend, util::{math_config::{Float2D, Float3D}, number::Nu, vector::Ve}, volume::{remove_trait::VolumeRemove, VolumeQureyPosValid}, voxel::palette::shared::SharedPalette};
use std::fmt::Debug;

pub trait IT: Debug + Copy + Eq + Default + Sync + Send {}
pub trait BU: Debug + Clone + Default + Sync + Send {}

pub trait ModelGenerationTypes: Debug + Clone + Default + Sync + Send  {
    type Identifier: IT; 
    type UndoData: BU;
    type Volume: VolumeQureyPosValid<Vec3A, f32, 3> + Debug + Clone + Sync + Send ;
    type Volume2D: VolumeQureyPosValid<Vec2, f32, 2> + Debug + Clone + Sync + Send ;
}

pub trait Model: Sized + Sync + Send {
    type GenerationTypes: ModelGenerationTypes;
    type UpdateData: Sync + Send + Debug;

    fn new(palette: &mut SharedPalette, scene: &SceneWorkerSend, change: &ModelChangeSender<Self::GenerationTypes>) 
        -> impl std::future::Future<Output = OctaResult<Self>> + std::marker::Send;

    fn update(&mut self, update_data: Self::UpdateData, change: &ModelChangeSender<Self::GenerationTypes>) 
        -> impl std::future::Future<Output = OctaResult<()>> + std::marker::Send;

    fn tick(&mut self, scene: &SceneWorkerSend, change: &ModelChangeSender<Self::GenerationTypes>) 
        -> impl std::future::Future<Output = OctaResult<bool>> + std::marker::Send;

}
