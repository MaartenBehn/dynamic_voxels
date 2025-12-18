use egui_snarl::InPinId;

use crate::{model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, graph::ComposerGraph, nodes::ComposeNode}, data_types::{data_type::ComposeDataType, number::NumberTemplate}, examples::compose_island::TemplateValue, template::{Template, update::MakeTemplateData}}, util::{number::Nu, vector::Ve}, voxel::palette::Palette};

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposerGraph<V2, V3, T, B> { 
    pub fn make_material(
        &self, 
        original_node: &ComposeNode<B::ComposeType>, 
        in_index: usize, 
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> u8 {
        match &original_node.inputs[in_index].data_type {
            ComposeDataType::Material(color) => data.palette.get_index_simple_color(*color).unwrap(),
            _ => unreachable!()
        }
    }
}






