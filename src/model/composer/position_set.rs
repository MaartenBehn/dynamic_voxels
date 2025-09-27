use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::{Either, Itertools};
use octa_force::glam::{ivec2, IVec2, IVec3, Vec2, Vec3A};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::traits::ModelGenerationTypes, util::{math_config::{MC}, number::Nu, vector::Ve}};

use super::{build::BS, collapse::collapser::{CollapseNode, CollapseNodeKey, Collapser}, data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeType}, number::NumberTemplate, template::{ComposeTemplate, TemplateIndex, TemplateNode}, ModelComposer};
use crate::util::vector;
use crate::util::math_config;

#[derive(Debug, Clone)]
pub enum PositionSetTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    Hook(TemplateIndex),
    T2Dto3D(PositionSet2DTo3DTemplate<V2, V3, T>),
}

#[derive(Debug, Clone)]
pub struct PositionSet2DTo3DTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    p2d: Box<PositionSetTemplate<V2, V3, T>>,
    z: NumberTemplate<V2, V3, T>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {  
    pub fn make_position_set(&self, pin: OutPinId, template: &ComposeTemplate<V2, V3, T, B>) -> PositionSetTemplate<V2, V3, T> {

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::TemplatePositionSet2D => PositionSetTemplate::Hook(template.get_index_by_out_pin(pin)),
            ComposeNodeType::TemplatePositionSet3D => PositionSetTemplate::Hook(template.get_index_by_out_pin(pin)),
            ComposeNodeType::PositionSet2DTo3D => {
                let t2dto3d = PositionSet2DTo3DTemplate {
                    p2d: Box::new(self.make_position_set(self.get_input_pin_by_type(node, ComposeDataType::PositionSet2D), template)), 
                    z: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template),                
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
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex>  {
        match self {
            PositionSetTemplate::Hook(i) => vec![*i],
            PositionSetTemplate::T2Dto3D(template) => {
                template.p2d.get_dependend_template_nodes()
                    .chain(template.z.get_dependend_template_nodes())
                    .collect_vec()
            },
        }.into_iter()
    }

    pub fn get_value<V: Ve<T, D>, B: BS<V2, V3, T>, const D: usize>(
        &self, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> impl Iterator<Item = V> {
        match self {
            PositionSetTemplate::Hook(i) => Either::Left(collapser.get_dependend_position_set(*i, depends, collapser)),
            PositionSetTemplate::T2Dto3D(template) => {
                assert_eq!(3, D);

                let z = template.z.get_value(depends, collapser);
                let points = template.p2d.get_value::<V2, B, 2>(depends, collapser)
                    .map(move |v| {
                        let arr = v.to_array();
                        
                        // Safety: D is 3
                        let a = [arr[0], arr[1], z];
                        let b = unsafe { ArrayUnion { a }.b };
                        V::new(b)
                    }).collect_vec();

                Either::Right(points.into_iter())
            },
        }
    }
}
