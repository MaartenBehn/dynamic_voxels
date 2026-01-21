use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::{iproduct, Either, Itertools};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::{GetNewChildrenData, GetValueData}, collapser::{CollapseChildKey, CollapseNodeKey, Collapser}}, composer::{ModelComposer, nodes::{ComposeNode, ComposeNodeType}}, data_types::data_type::T, template::{self, Template, TemplateIndex, update::MakeTemplateData, value::TemplateValue}}, util::{iter_merger::IM2, math_config::MC, number::Nu, vector::Ve}};

use crate::util::vector;
use crate::util::math_config;

use super::{data_type::ComposeDataType, number::{Hook, NumberTemplate, ValueIndexNumber}, position_space::ValueIndexPositionSpace};

pub type ValueIndexPositionSet = usize;
pub type ValueIndexPositionSet2D = usize;
pub type ValueIndexPositionSet3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionSetTemplate {
    All(ValueIndexPositionSpace),
}
 
impl PositionSetTemplate { 
    pub fn get_value<V: Ve<T, D>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser,
        template: &Template
    ) -> (Vec<V>, bool) {
        match self {
            PositionSetTemplate::All(space) => {
                template.get_position_space_value(*space)
                    .get_value(get_value_data, collapser, template)
            },
        }
    }
}
