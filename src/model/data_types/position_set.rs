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
pub enum PositionSetTemplate {
    Hook(Hook),
    T2Dto3D(PositionSet2DTo3DTemplate),
}

#[derive(Debug, Clone, Copy)]
pub struct PositionSet2DTo3DTemplate {
    p2d: ValueIndexPositionSet2D,
    z: ValueIndexNumber,
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
                let template_index = data.template.get_index_by_out_pin(pin);
                data.depends.push(template_index);

                ComposeTemplateValue::PositionSet2D(PositionSetTemplate::Hook(Hook {
                    template_index,
                    loop_cut: false,
                }))
            },
            ComposeNodeType::TemplatePositionSet3D => {
                let template_index = data.template.get_index_by_out_pin(pin);
                data.depends.push(template_index);

                ComposeTemplateValue::PositionSet3D(PositionSetTemplate::Hook(Hook {
                    template_index,
                    loop_cut: false,
                }))
            },
            ComposeNodeType::PositionSet2DTo3D => {
                let t2dto3d = PositionSet2DTo3DTemplate {
                    p2d: self.make_position_set(self.get_input_remote_pin_by_index(node, 0), data), 
                    z: self.make_number(node, 1, data),                
                };
                ComposeTemplateValue::PositionSet3D(PositionSetTemplate::T2Dto3D(t2dto3d))
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

impl PositionSetTemplate {
    pub fn get_ammount_hook<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>>(&self, template: &ComposeTemplate<V2, V3, T, B>) -> TemplateIndex {
        match self {
            PositionSetTemplate::Hook(hook) => hook.template_index,
            PositionSetTemplate::T2Dto3D(set) 
                => template.get_position_set_value(set.p2d).get_ammount_hook(template),
        }
    }

    pub fn get_value<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> (impl Iterator<Item = V>, bool) {
        match self {
            PositionSetTemplate::Hook(hook) => {
                let (set, r) = collapser.get_dependend_position_set(hook.template_index, get_value_data);
                (IM2::A(set), r)
            },
            PositionSetTemplate::T2Dto3D(set) => {
                assert_eq!(3, D);

                let (v, r_0) = template.get_position_set_value(set.p2d)
                    .get_value::<V2, V2, V3, T, B, 2>(get_value_data, collapser, template);

                let (z, r_1) = template.get_number_value(set.z)
                    .get_value(get_value_data, collapser, template);

                let v = v.collect_vec();

                let v = iproduct!(v, z)
                    .map(|(v, z)| {
                        let arr = v.to_array();
                        // Safety: D is 3
                        let a = [arr[0], arr[1], z];
                        let b = unsafe { ArrayUnion { a }.b };
                        V::new(b)
                    });

                (IM2::B(v), r_0 || r_1)
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
            PositionSetTemplate::Hook(hook) => {
                let (v, r) = collapser.get_dependend_position(hook.template_index, get_value_data);
                (v.collect(), r)
            },
            PositionSetTemplate::T2Dto3D(set) => {
                assert_eq!(3, D);

                let (v, r_0) = template.get_position_set_value(set.p2d)
                    .get_child_value::<V, V2, V3, T, B, D>(get_value_data, collapser, template);

                let (z, r_1) = template.get_number_value(set.z)
                    .get_value(get_value_data, collapser, template);

                let v = iproduct!(v, z)
                    .map(|(v, z)| {
                        let arr = v.to_array();
                        // Safety: D is 3
                        let a = [arr[0], arr[1], z];
                        let b = unsafe { ArrayUnion { a }.b };
                        V::new(b)
                    });

                (v.collect(), r_0 || r_1)
            },
        }
    }
}
