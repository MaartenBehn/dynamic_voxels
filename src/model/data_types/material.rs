use egui_snarl::InPinId;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{graph::ComposerGraph, nodes::ComposeNode}, data_types::{data_type::ComposeDataType, number::NumberTemplate}, template::{Template, update::MakeTemplateData}}, util::{number::Nu, vector::Ve}, voxel::palette::Palette};

impl ComposerGraph { 
    pub fn make_material(
        &self, 
        original_node: &ComposeNode, 
        in_index: usize, 
        data: &mut MakeTemplateData,
    ) -> u8 {
        match &original_node.inputs[in_index].data_type {
            ComposeDataType::Material(color) => data.palette.get_index_simple_color(*color).unwrap(),
            _ => unreachable!()
        }
    }
}






