use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::{Either, Itertools};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::{GetNewChildrenData, GetValueData}, collapser::{CollapseChildKey, CollapseNodeKey, Collapser}}, composer::{build::BS, nodes::ComposeNodeType, template::{ComposeTemplate, MakeTemplateData, TemplateIndex}, ModelComposer}}, util::{math_config::MC, number::Nu, vector::Ve}};

use crate::util::vector;
use crate::util::math_config;

use super::{data_type::ComposeDataType, number::{Hook, NumberTemplate}};

#[derive(Debug, Clone)]
pub enum PositionSetTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    Hook(Hook),
    T2Dto3D(PositionSet2DTo3DTemplate<V2, V3, T>),
}

#[derive(Debug, Clone)]
pub struct PositionSet2DTo3DTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    p2d: Box<PositionSetTemplate<V2, V3, T>>,
    z: NumberTemplate<V2, V3, T>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {  
    pub fn make_position_set(
        &self, 
        pin: OutPinId, 
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> PositionSetTemplate<V2, V3, T> {

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");

        match &node.t {
            ComposeNodeType::TemplatePositionSet2D => {
                let template_index = data.template.get_index_by_out_pin(pin);
                data.depends.push(template_index);

                PositionSetTemplate::Hook(Hook {
                    template_index,
                    loop_cut: false,
                })
            },
            ComposeNodeType::TemplatePositionSet3D => {
                let template_index = data.template.get_index_by_out_pin(pin);
                data.depends.push(template_index);

                PositionSetTemplate::Hook(Hook {
                    template_index,
                    loop_cut: false,
                })
            },
            ComposeNodeType::PositionSet2DTo3D => {
                let t2dto3d = PositionSet2DTo3DTemplate {
                    p2d: Box::new(self.make_position_set(self.get_input_remote_pin_by_type(node, ComposeDataType::PositionSet2D), data)), 
                    z: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data),                
                };
                PositionSetTemplate::T2Dto3D(t2dto3d)
            },
            _ => unreachable!()
        }
    }
} 

union ArrayUnion<T: Nu, const D: usize> {
    a: [T; 3],
    b: [T; D],
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> PositionSetTemplate<V2, V3, T> {
    pub fn get_ammount_hook(&self) -> TemplateIndex {
        match self {
            PositionSetTemplate::Hook(hook) => hook.template_index,
            PositionSetTemplate::T2Dto3D(template) => template.p2d.get_ammount_hook(),
        }
    }

    pub fn get_new_children<B: BS<V2, V3, T>>(
        &self, 
        get_data: GetNewChildrenData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (impl Iterator<Item = (CollapseNodeKey, CollapseChildKey)>, bool) {
        match self {
            PositionSetTemplate::Hook(hook) => {
                let (keys, r) = collapser.get_dependend_new_children(hook.template_index, get_data);
                (Either::Left(keys), r)
            },
            PositionSetTemplate::T2Dto3D(template) => {
                let (keys, r) = template.p2d.get_new_children::<B>(get_data, collapser);

                // To break type recursion.
                // Hopefully this gets optimized away.
                let keys = keys.collect_vec();
                let keys = keys.into_iter();

                (Either::Right(keys), r)
            },
        }
    }

    pub fn is_child_valid<B: BS<V2, V3, T>>(
        &self, 
        index: CollapseNodeKey,
        child_index: CollapseChildKey,
        collapser: &Collapser<V2, V3, T, B>
    ) -> bool {
        match self {
            PositionSetTemplate::Hook(hook) => {
                collapser.is_child_valid(index, child_index)
            },
            PositionSetTemplate::T2Dto3D(template) => {
                template.p2d.is_child_valid::<B>(index, child_index, collapser)
            },
        }
    }

    pub fn get_value<V: Ve<T, D>, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (impl Iterator<Item = V>, bool) {
        match self {
            PositionSetTemplate::Hook(hook) => {
                let (set, r) = collapser.get_dependend_position_set(hook.template_index, get_value_data);
                (Either::Left(set), r)
            },
            PositionSetTemplate::T2Dto3D(template) => {
                assert_eq!(3, D);

                let (z, r_0) = template.z.get_value(get_value_data, collapser);
                let (points, r_1) = template.p2d.get_value::<V2, B, 2>(get_value_data, collapser);

                // TODO fix allocation
                let points = points.map(move |v| {
                        let arr = v.to_array();
                        
                        // Safety: D is 3
                        let a = [arr[0], arr[1], z];
                        let b = unsafe { ArrayUnion { a }.b };
                        V::new(b)
                    }).collect_vec();

                (Either::Right(points.into_iter()), r_0 || r_1)
            },
        }
    }

    pub fn get_child_value<V: Ve<T, D>, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (V, bool) {
        match self {
            PositionSetTemplate::Hook(hook) => {
                let (v, r) = collapser.get_dependend_position(hook.template_index, get_value_data);
                (v, r)
            },
            PositionSetTemplate::T2Dto3D(template) => {
                assert_eq!(3, D);

                let (z, r_0) = template.z.get_value(get_value_data, collapser);
                let (v, r_1) = template.p2d.get_child_value::<V2, B, 2>(get_value_data, collapser);
                let arr = v.to_array();

                // Safety: D is 3
                let a = [arr[0], arr[1], z];
                let b = unsafe { ArrayUnion { a }.b };

                let v = V::new(b);

                (v, r_0 || r_1)
            },
        }
    }

    pub fn cut_loop(&mut self, to_index: usize) {
        match self {
            PositionSetTemplate::Hook(hook) => {
                hook.loop_cut |= to_index == hook.template_index;
            },
            PositionSetTemplate::T2Dto3D(template) => {
                template.p2d.cut_loop(to_index);
                template.z.cut_loop(to_index);
            },
        }
    }
}
