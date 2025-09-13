use egui_snarl::{InPinId, NodeId, OutPinId};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::traits::ModelGenerationTypes, util::{math_config::{MC}, number::Nu, vector::Ve}};

use super::{build::BS, collapse::collapser::{CollapseNode, CollapseNodeKey, Collapser}, data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, template::{ComposeTemplate, TemplateIndex, TemplateNode}, ModelComposer};
use crate::util::vector;
use crate::util::math_config;

#[derive(Debug, Clone, Copy)]
pub struct PositionSetTemplate {
    template_index: TemplateIndex,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {  
    pub fn make_position_set(&self, pin: OutPinId, template: &ComposeTemplate<V2, V3, T, B>) -> PositionSetTemplate {
        PositionSetTemplate{ 
            template_index: template.get_index_by_out_pin(pin)
        }
    }
} 

impl PositionSetTemplate {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex>  {
        Some(self.template_index).into_iter()
    }

    pub fn get_value<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> impl Iterator<Item = V> {
        collapser.get_dependend_position_set(self.template_index, depends, collapser)
    }
}
