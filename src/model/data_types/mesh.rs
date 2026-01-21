use std::time::Instant;

use octa_force::{glam::{Mat4, Quat, Vec3}, log::info};
use smallvec::SmallVec;

use crate::{VOXELS_PER_SHADER_UNIT, csg::csg_tree::tree::CSGTree, mesh::Mesh, model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{graph::ComposerGraph, nodes::ComposeNode, output_state::OutputState}, data_types::{data_type::{T, V3}, position::ValueIndexPosition, volume::ValueIndexVolume}, template::{Template, update::MakeTemplateData, value::TemplateValue}}, scene::SceneObjectKey, volume::VolumeBounds, voxel::dag64::DAG64EntryKey};

pub type ValueIndexVoxels = usize;

#[derive(Debug, Clone, Copy)]
pub struct MeshTemplate {
    pub pos: ValueIndexPosition,
    pub volume: ValueIndexVolume,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MeshCollapserData {
    scene_key: SceneObjectKey,
    dag_key: DAG64EntryKey,
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
        &self,
        volume: CSGTree<u8, V3, T, 3>,
        pos: V3,
        state: &mut OutputState,
    ) -> MeshCollapserData {

        dbg!(&volume);

        let now = Instant::now();

        let mesh = Mesh::from_volume(&volume);

        let elapsed = now.elapsed();
        info!("Mesh Build took: {:?}", elapsed);

        dbg!(&mesh);

        MeshCollapserData::default()
    }
}

