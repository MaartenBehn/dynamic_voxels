use std::time::Instant;

use octa_force::{glam::{Mat4, Quat, Vec3}, log::info};
use slotmap::Key;
use smallvec::SmallVec;

use crate::{VOXELS_PER_SHADER_UNIT, csg::csg_tree::tree::CSGTree, mesh::{Mesh, scene::SceneMeshKey}, model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{graph::ComposerGraph, nodes::ComposeNode, output_state::OutputState}, data_types::{data_type::{T, V3}, position::ValueIndexPosition, volume::ValueIndexVolume}, template::{Template, update::MakeTemplateData, value::TemplateValue}},  volume::VolumeBounds, voxel::dag64::DAG64EntryKey};

pub type ValueIndexVoxels = usize;

#[derive(Debug, Clone, Copy)]
pub struct MeshTemplate {
    pub pos: ValueIndexPosition,
    pub volume: ValueIndexVolume,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MeshCollapserData {
    mesh_key: SceneMeshKey,
}

impl ComposerGraph { 
    pub fn make_mesh(
        &self, 
        node: &ComposeNode, 
        data: &mut MakeTemplateData,
    ) -> ValueIndexVoxels {
        let node_data = self.start_template_node(node, data);

        let volume = self.make_volume(node, 0, data);
        let pos = self.make_position(node, 1, data);    

        let value = TemplateValue::Mesh(MeshTemplate { volume, pos });

        let value_index = data.add_value(value);
        node_data.finish_template_node(value_index, data);

        value_index
    }
}


impl MeshCollapserData {
    pub async fn update(
        &mut self,
        volume: CSGTree<u8, V3, T, 3>,
        pos: V3,
        state: &mut OutputState,
    ) {

        let now = Instant::now();

        let mesh = Mesh::from_volume(&volume);

        let elapsed = now.elapsed();
        info!("Mesh Build took: {:?}", elapsed);

        if !self.mesh_key.is_null() {
            state.mesh_scene.remove(self.mesh_key);
        }

        let mesh_key = state.mesh_scene.add(
            mesh,
            Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                Vec3::from(pos) / VOXELS_PER_SHADER_UNIT as f32
            ),
        ).result_async().await; 
        
        self.mesh_key = mesh_key;
    }
}

