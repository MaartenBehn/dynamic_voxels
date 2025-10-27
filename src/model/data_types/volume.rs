use egui_snarl::OutPinId;
use itertools::{iproduct, Itertools};
use smallvec::SmallVec;

use crate::{csg::{csg_tree::tree::CSGTree, Base}, model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::ComposeNodeType, ModelComposer}, template::{update::MakeTemplateData, value::ComposeTemplateValue, ComposeTemplate}}, util::{iter_merger::IM5, number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, number::{NumberTemplate, ValueIndexNumber}, position::{PositionTemplate, ValueIndexPosition}, position_set::{PositionSetTemplate, ValueIndexPositionSet}};

pub type ValueIndexVolume = usize;
pub type ValueIndexVolume2D = usize;
pub type ValueIndexVolume3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum VolumeTemplate {
    Sphere {
        pos: ValueIndexPosition,
        size: ValueIndexNumber,
    },
    Box {
        pos: ValueIndexPosition,
        size: ValueIndexNumber,
    },
    Union {
        a: ValueIndexVolume,
        b: ValueIndexVolume,
    },
    Cut {
        base: ValueIndexVolume,
        cut: ValueIndexVolume,
    },
    SphereUnion {
        position_set: ValueIndexPositionSet,
        size: ValueIndexNumber,
    },
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_volume(
        &self, 
        pin: OutPinId,
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> ValueIndexVolume {
        if let Some(value_index) = data.value_per_node_id.get_value(pin.node) {
            return value_index;
        }

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        let value = match &node.t {
            ComposeNodeType::Sphere => ComposeTemplateValue::Volume3D(VolumeTemplate::Sphere { 
                pos: self.make_position(node, 0, data),
                size: self.make_number(node, 1, data),
            }),
            ComposeNodeType::Box2D => ComposeTemplateValue::Volume2D(VolumeTemplate::Box { 
                pos: self.make_position(node, 0, data),
                size: self.make_position(node, 1, data),
            }),
            ComposeNodeType::Box3D => ComposeTemplateValue::Volume3D(VolumeTemplate::Box { 
                pos: self.make_position(node, 0, data),
                size: self.make_position(node, 1, data),
            }),
            ComposeNodeType::UnionVolume2D => ComposeTemplateValue::Volume2D(VolumeTemplate::Union {
                a: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                b: self.make_volume(self.get_input_remote_pin_by_index(node, 1), data),
            }),
            ComposeNodeType::UnionVolume3D => ComposeTemplateValue::Volume3D(VolumeTemplate::Union {
                a: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                b: self.make_volume(self.get_input_remote_pin_by_index(node, 1), data),
            }),
            ComposeNodeType::CutVolume2D => ComposeTemplateValue::Volume2D(VolumeTemplate::Cut {
                base: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                cut: self.make_volume(self.get_input_remote_pin_by_index(node, 1), data),
            }),
            ComposeNodeType::CutVolume3D => ComposeTemplateValue::Volume3D(VolumeTemplate::Cut {
                base: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                cut: self.make_volume(self.get_input_remote_pin_by_index(node, 1), data),
            }),
            ComposeNodeType::CircleUnion => ComposeTemplateValue::Volume3D(VolumeTemplate::SphereUnion { 
                position_set: self.make_position_set(self.get_input_remote_pin_by_index(node, 0), data), 
                size: self.make_number(node, 1, data) 
            }),
            ComposeNodeType::SphereUnion => ComposeTemplateValue::Volume3D(VolumeTemplate::SphereUnion { 
                position_set: self.make_position_set(self.get_input_remote_pin_by_index(node, 0), data), 
                size: self.make_number(node, 1, data) 
            }),
            _ => unreachable!(),
        };

        data.set_value(pin.node, value)
    }
}

impl VolumeTemplate {  
    pub fn get_value<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, M: Base, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>,
        mat: M,
    ) -> (CSGTree<M, V, T, D>, bool) {
        let mut tree = CSGTree::default();

        let (roots, r) = self.get_value_inner(get_value_data, collapser, template, mat, &mut tree);
        
        if roots.len() == 1 {
            tree.set_root(roots[0]);
        } else {
            let root = tree.add_union_node(roots); 
            tree.set_root(root);
        };

        (tree, r)
    }

    pub fn get_value_inner<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, M: Base, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>,
        mat: M,
        tree: &mut CSGTree<M, V, T, D>,
    ) -> (Vec<usize>, bool) {

        match &self {
            VolumeTemplate::Sphere { pos, size } => {
                
                let (pos, r_0) = template.get_position_value::<V, D>(*pos)
                    .get_value(get_value_data, collapser, template);

                let (size, r_1) = template.get_number_value(*size)
                    .get_value(get_value_data, collapser, template);

                let mut roots = vec![];
                for (pos, size) in iproduct!(pos, size) {
                    roots.push(tree.add_sphere(pos.to_vecf(), size.to_f32(), mat));
                }

                (roots, r_0 || r_1)            
            },
            VolumeTemplate::Box { pos, size } => {
                let (pos, r_0) = template.get_position_value::<V, D>(*pos)
                    .get_value(get_value_data, collapser, template);

                let (size, r_1) = template.get_position_value::<V, D>(*size)
                    .get_value(get_value_data, collapser, template);

                let mut roots = vec![];
                for (pos, size) in iproduct!(pos, size) {
                    roots.push(tree.add_box(pos.to_vecf(), size.to_vecf(), mat));
                }

                (roots, r_0 || r_1)
            },
            VolumeTemplate::Union { a, b } => {
                let (mut a, r_0) = template.get_volume_value(*a)
                    .get_value_inner(get_value_data, collapser, template, mat, tree);
          
                let (mut b, r_1) = template.get_volume_value(*b)
                    .get_value_inner(get_value_data, collapser, template, mat, tree);

                a.append(&mut b);

                (a, r_0 || r_1)
            },
            VolumeTemplate::Cut { base, cut } => {
                let (mut base, r_0) = template.get_volume_value(*base)
                    .get_value_inner(get_value_data, collapser, template, mat, tree);
          
                let (mut cut, r_1) = template.get_volume_value(*cut)
                    .get_value_inner(get_value_data, collapser, template, mat, tree);

                if base.is_empty() {
                    return (vec![], r_0 || r_1);
                }

                if cut.is_empty() {
                    return (base, r_0 || r_1);
                }

                let base = if base.len() == 1 {
                    base[0]
                } else {
                    tree.add_union_node(base)
                };
                
                let cut = if cut.len() == 1 {
                    cut[0]
                } else {
                    tree.add_union_node(cut)
                };

                let root = tree.add_cut_node(base, cut);

                (vec![root], r_0 || r_1)
            },
            VolumeTemplate::SphereUnion { position_set, size } => {

                let (set, r_1) = template.get_position_set_value(*position_set)
                    .get_value::<V, V2, V3, T, B, D>(get_value_data, collapser, template);
                let (size, r_0) = template.get_number_value(*size)
                    .get_value(get_value_data, collapser, template);

                let mut roots = vec![];
                for (pos, size) in iproduct!(set, size) {
                    roots.push(tree.add_sphere(pos.to_vecf(), size.to_f32(), mat));
                }

                (roots, r_0 || r_1)
            },
        }
    }
}
