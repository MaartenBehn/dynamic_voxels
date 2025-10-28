use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::{iproduct, Either, Itertools};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::{GetNewChildrenData, GetValueData}, collapser::{CollapseChildKey, CollapseNodeKey, Collapser}}, composer::{build::BS, nodes::ComposeNodeType, ModelComposer}, template::{self, update::MakeTemplateData, value::ComposeTemplateValue, ComposeTemplate, TemplateIndex}}, util::{iter_merger::IM2, math_config::MC, number::Nu, vector::Ve}};

use crate::util::vector;
use crate::util::math_config;

use super::{data_type::ComposeDataType, number::{Hook, NumberTemplate, ValueIndexNumber}};

pub type ValueIndexPositionSet = usize;
pub type ValueIndexPositionSet2D = usize;
pub type ValueIndexPositionSet3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionPairSetTemplate {
    ByDistance((Hook)),
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {  
    pub fn make_position_pair_set(
        &self, 
        pin: OutPinId, 
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> ValueIndexPositionSet {
        if let Some(value_index) = data.value_per_node_id.get_value(pin.node) {
            return value_index;
        }

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");

        let value = match &node.t {
            ComposeNodeType::PositionPairSet2D => {
                let template_index = data.template.get_index_by_out_pin(pin);
                data.depends.push(template_index);

                ComposeTemplateValue::PositionPairSet2D(PositionPairSetTemplate::Hook(Hook {
                    template_index,
                    loop_cut: false,
                }))
            },
            ComposeNodeType::PositionPairSet3D => {
                let template_index = data.template.get_index_by_out_pin(pin);
                data.depends.push(template_index);

                ComposeTemplateValue::PositionPairSet3D(PositionPairSetTemplate::Hook(Hook {
                    template_index,
                    loop_cut: false,
                }))
            },
            _ => unreachable!()
        };

        data.set_value(pin.node, value)
    }
} 

union ArrayUnion<T: Nu, const D: usize> {
    a: [T; 3],
    b: [T; D],
}

impl PositionPairSetTemplate {
    pub fn get_ammount_hook<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>>(&self, template: &ComposeTemplate<V2, V3, T, B>) -> TemplateIndex {
        match self {
            PositionPairSetTemplate::Hook(hook) => hook.template_index,
        }
    }

    pub fn get_value<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> (impl Iterator<Item = (V, V)>, bool) {
        match self {
            PositionPairSetTemplate::Hook(hook) => {
                let (set, r) = collapser.get_dependend_position_pair_set(hook.template_index, get_value_data);
                (IM2::A(set), r)
            },
        }
    }

    pub fn get_child_value<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> (SmallVec<[V; 1]>, bool) {
        match self {
            PositionPairSetTemplate::Hook(hook) => {
                let (v, r) = collapser.get_dependend_position(hook.template_index, get_value_data);
                (v.collect(), r)
            },
        }
    }
}
