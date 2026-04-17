use std::time::Instant;

use octa_force::{glam::{Mat4, Quat, Vec3}, log::info};
use slotmap::Key;
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::GetValueData, collapser::{CollapseValueT, Collapser}, template_changed::MatchValueData}, composer::{graph::ComposerGraph, make_template::MakeTemplateData, nodes::ComposeNode, output_state::OutputState}, data_types::{data_type::TemplateValue, position::{ValueIndexPosition, ValueIndexPosition3D}, volume::ValueIndexVolume}, template::Template}, scene::worker::SceneObjectKey, util::{default_types::{V3, Volume}, shader_constants::VOXELS_PER_SHADER_UNIT}, volume::VolumeBounds, voxel::dag64::{entry::DAG64EntryKey, lod_heuristic::LODHeuristicNone}};

pub type ValueIndexVoxels = usize;

#[derive(Debug, Clone, Copy)]
pub struct VoxelValue {
    pub pos: ValueIndexPosition3D,
    pub volume: ValueIndexVolume,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VoxelCollapserData {
    scene_key: SceneObjectKey,
    dag_key: DAG64EntryKey,
}

impl ComposerGraph { 
    pub fn make_voxels(
        &self, 
        node: &ComposeNode, 
        data: &mut MakeTemplateData,
    ) -> ValueIndexVoxels {
        let node_data = self.start_template_node(node, data);

        let volume = self.make_volume(node, 0, data);
        let pos = self.make_position(node, 1, data);    

        let value = TemplateValue::Voxels(VoxelValue { volume, pos });

        let value_index = data.add_value(value);
        node_data.finish_template_node(value_index, data);

        value_index
    }
}

impl VoxelValue {
    pub fn match_value(
        &self, 
        other: &VoxelValue,
        data: MatchValueData
    ) -> bool {

        data.match_two_volumes(self.volume, other.volume)
        && data.match_two_positions3d(self.pos, other.pos)
    }
}

impl VoxelCollapserData {
    pub async fn update(
        &mut self,
        volume: Volume,
        pos: V3,
        state: &mut OutputState,
    ) { 
        self.on_delete(state);

        let now = Instant::now();
        info!("Update Voxel Object at: {:?}", now);
        return;

        self.scene_key = state.scene.add_object(
            Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                Vec3::from(pos) / VOXELS_PER_SHADER_UNIT as f32
            ), 
            volume,
        ).result_async().await;
    } 
}

impl CollapseValueT for VoxelCollapserData {
    fn on_delete(&self, state: &mut OutputState) {

        if !self.scene_key.is_null() {
            state.scene.remove_object(self.scene_key);
        }
    }
}

