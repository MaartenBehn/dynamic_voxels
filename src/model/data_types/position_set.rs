use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::{iproduct, Either, Itertools};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::{GetNewChildrenData, GetValueData}, collapser::{CollapseChildKey, CollapseNodeKey, Collapser}}, composer::{build::BS, nodes::ComposeNodeType, ModelComposer}, template::{self, update::MakeTemplateData, value::ComposeTemplateValue, ComposeTemplate, TemplateIndex}}, util::{iter_merger::IM2, math_config::MC, number::Nu, vector::Ve}};

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

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {  
    pub fn make_position_set(
        &self, 
        pin: OutPinId, 
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> ValueIndexPositionSet {
        if let Some(value_index) = data.value_per_node_id.get_value(pin.node) {
            return value_index;
        }

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");

        let value = match &node.t {
            ComposeNodeType::TemplatePositionSet2D => {
                let space = self.make_pos_space(pin, data); 
                ComposeTemplateValue::PositionSet2D(PositionSetTemplate::All(space))
            },
            ComposeNodeType::TemplatePositionSet3D => {
                let space = self.make_pos_space(pin, data); 
                ComposeTemplateValue::PositionSet2D(PositionSetTemplate::All(space))
            },
            _ => unreachable!()
        };

        data.set_value(pin.node, value)
    }
} 

impl PositionSetTemplate { 
    pub fn get_value<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> (Vec<V>, bool) {
        match self {
            PositionSetTemplate::All(space) => {
                template.get_position_space_value(*space)
                    .get_value(get_value_data, collapser, template)
            },
        }
    }
}
