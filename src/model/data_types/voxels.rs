use std::time::Instant;

use octa_force::{glam::{Mat4, Quat, Vec3}, log::info};
use smallvec::SmallVec;

use crate::{VOXELS_PER_SHADER_UNIT, csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{graph::ComposerGraph, nodes::ComposeNode, output_state::OutputState}, data_types::{data_type::{T, V3}, position::ValueIndexPosition, volume::ValueIndexVolume}, template::{Template, update::MakeTemplateData, value::TemplateValue}}, scene::SceneObjectKey, volume::VolumeBounds, voxel::dag64::DAG64EntryKey};

pub type ValueIndexVoxels = usize;

#[derive(Debug, Clone, Copy)]
pub struct VoxelTemplate {
    pub pos: ValueIndexPosition,
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

        let value = TemplateValue::Voxels(VoxelTemplate { volume, pos });

        let value_index = data.add_value(value);
        node_data.finish_template_node(value_index, data);

        value_index
    }
}


impl VoxelCollapserData {
    pub async fn update(
        &self,
        volume: CSGTree<u8, V3, T, 3>,
        pos: V3,
        state: &mut OutputState,
    ) -> VoxelCollapserData {

        dbg!(&volume);

        let now = Instant::now();
        let dag_key = state.dag.add_aabb_query_volume(&volume).expect("Could not add DAG Entry!");
        let elapsed = now.elapsed();
        info!("Voxel DAG Build took: {:?}", elapsed);

        //delete_object(args.collapse_value, state);
        let scene_key = state.scene.add_dag_object(
            Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                Vec3::from(pos) / VOXELS_PER_SHADER_UNIT as f32
            ), 
            state.scene_dag_key,
            state.dag.get_entry(dag_key),
        ).result_async().await;

        VoxelCollapserData { 
            scene_key, 
            dag_key 
        }
    }
}

