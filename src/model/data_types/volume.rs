use std::{any::TypeId, marker::PhantomData, mem::ManuallyDrop};

use egui_snarl::OutPinId;
use itertools::{iproduct, Itertools};
use smallvec::SmallVec;

use crate::{csg::{Base, csg_tree::tree::CSGTree}, model::{collapse::{add_nodes::GetValueData, collapser::Collapser, template_changed::MatchValueData}, composer::{ModelComposer, graph::ComposerGraph, make_template::MakeTemplateData, nodes::ComposeNode}, data_types::data_type::{ComposeNodeType, T, TemplateValue, V3}, template::Template}, util::{iter_merger::IM5, number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, number::{NumberValue, ValueIndexNumber}, position::{PositionValue, ValueIndexPosition}, position_set::{PositionSetValue, ValueIndexPositionSet}};

pub type ValueIndexVolume = usize;
pub type ValueIndexVolume2D = usize;
pub type ValueIndexVolume3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum VolumeValue {
    Sphere {
        pos: ValueIndexPosition,
        size: ValueIndexNumber,
    },
    Disk {
        pos: ValueIndexPosition,
        size: ValueIndexNumber,
        height: ValueIndexNumber,
    },
    Box {
        pos: ValueIndexPosition,
        size: ValueIndexPosition,
    },
    Union {
        a: ValueIndexVolume,
        b: ValueIndexVolume,
    },
    Cut {
        base: ValueIndexVolume,
        cut: ValueIndexVolume,
    },
    Material {
        mat: u8,
        child: ValueIndexVolume,
    },
}

union TreeUnion<'a, MA, MB, VA: Ve<T, DA>, VB: Ve<T, DB>, T: Nu, const DA: usize, const DB: usize> {
    a: &'a mut CSGTree<MA, VA, T, DA>,
    b: &'a mut CSGTree<MB, VB, T, DB>,
    p: PhantomData<T>,
}

union PositionUnion<VA: Ve<T, DA>, VB: Ve<T, DB>, T: Nu, const DA: usize, const DB: usize> {
    a: VA,
    b: VB,
    p: PhantomData<T>,
}

impl ComposerGraph {
    pub fn make_volume(
        &self, 
        original_node: &ComposeNode, 
        in_index: usize, 
        data: &mut MakeTemplateData,
    ) -> ValueIndexVolume {
        let node_id = self.get_input_remote_node_id(original_node, in_index);
        
        if let Some(value_index) = data.get_value_index_from_node_id(node_id) {
            data.add_depends_of_value(value_index);
            return value_index;
        }

        let node = self.snarl.get_node(node_id).expect("Node of remote not found");
        let value = match &node.t {
            ComposeNodeType::Circle => TemplateValue::Volume2D(VolumeValue::Sphere { 
                pos: self.make_position(node, 0, data),
                size: self.make_number(node, 1, data),
            }),
            ComposeNodeType::Sphere => TemplateValue::Volume3D(VolumeValue::Sphere { 
                pos: self.make_position(node, 0, data),
                size: self.make_number(node, 1, data),
            }),
            ComposeNodeType::Disk => TemplateValue::Volume3D(VolumeValue::Disk { 
                pos: self.make_position(node, 0, data),
                size: self.make_number(node, 1, data),
                height: self.make_number(node, 2, data),
            }),
            ComposeNodeType::Box2D => TemplateValue::Volume2D(VolumeValue::Box { 
                pos: self.make_position(node, 0, data),
                size: self.make_position(node, 1, data),
            }),
            ComposeNodeType::Box3D => TemplateValue::Volume3D(VolumeValue::Box { 
                pos: self.make_position(node, 0, data),
                size: self.make_position(node, 1, data),
            }),
            ComposeNodeType::UnionVolume2D => TemplateValue::Volume2D(VolumeValue::Union {
                a: self.make_volume(node, 0, data),
                b: self.make_volume(node, 1, data),
            }),
            ComposeNodeType::UnionVolume3D => TemplateValue::Volume3D(VolumeValue::Union {
                a: self.make_volume(node, 0, data),
                b: self.make_volume(node, 1, data),
            }),
            ComposeNodeType::CutVolume2D => TemplateValue::Volume2D(VolumeValue::Cut {
                base: self.make_volume(node, 0, data),
                cut: self.make_volume(node, 1, data),
            }),
            ComposeNodeType::CutVolume3D => TemplateValue::Volume3D(VolumeValue::Cut {
                base: self.make_volume(node, 0, data),
                cut: self.make_volume(node, 1, data),
            }),
            ComposeNodeType::VolumeMaterial2D => TemplateValue::Volume2D(VolumeValue::Material {
                child: self.make_volume(node, 0, data),
                mat: self.make_material(node, 1, data),
            }),
            ComposeNodeType::VolumeMaterial3D => TemplateValue::Volume3D(VolumeValue::Material {
                child: self.make_volume(node, 0, data),
                mat: self.make_material(node, 1, data),
            }),
            _ => unreachable!(),
        };

        data.set_value(node_id, value)
    }
}

impl VolumeValue { 
    pub fn match_value(
        &self, 
        other: &VolumeValue,
        data: MatchValueData
    ) -> bool {

        match self {
            VolumeValue::Sphere { pos: p1, size: n1 } => match other {
                VolumeValue::Sphere { pos: p2, size: n2 } => {
                    data.match_two_positions_check(*p1, *p2)
                    && data.match_two_numbers(*n1, *n2)
                },
                _ => false,
            },
            VolumeValue::Disk { pos: p1, size: size1, height: height1 } => match other {
                VolumeValue::Disk { pos: p2, size: size2, height: height2 } => {
                    data.match_two_positions_check(*p1, *p2)
                    && data.match_two_numbers(*size1, *size2)
                    && data.match_two_numbers(*height1, *height2)
                }, 
                _ => false,
            },
            VolumeValue::Box { pos: p1, size: size1 } => match other {
                VolumeValue::Box { pos: p2, size: size2 } => {
                    data.match_two_positions_check(*p1, *p2)
                    && data.match_two_positions_check(*size1, *size2)
                },
                _ => false,
            },
            VolumeValue::Union { a: a1, b: b1 } => match other {
                VolumeValue::Union { a: a2, b: b2 } => {
                    data.match_two_volumes(*a1, *a2)
                    && data.match_two_volumes(*b1, *b2)
                }, 
                _ => false,
            },
            VolumeValue::Cut { base: base1, cut: cut1 } => match other {
                VolumeValue::Cut { base: base2, cut: cut2 } => {
                    data.match_two_volumes(*base1, *base2)
                    && data.match_two_volumes(*cut1, *cut2)
                },
                _ => false,
            },
            VolumeValue::Material { mat: mat1, child: child1 } => match other {
                VolumeValue::Material { mat: mat2, child: child2 } => {
                    mat1 == mat2 
                    && data.match_two_volumes(*child1, *child2)
                },
                _ => false
            },
        }
    }

    pub fn get_value<V: Ve<T, D>, M: Base, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser,
    ) -> (CSGTree<M, V, T, D>, bool) {
        let mut tree = CSGTree::default();

        let (roots, r) = self.get_value_inner(get_value_data, collapser, M::base(), &mut tree);
        
        if roots.len() == 1 {
            tree.set_root(roots[0]);
        } else if roots.len() > 1 {
            let root = tree.add_union_node(roots); 
            tree.set_root(root);
        };

        (tree, r)
    }

    pub fn get_value_inner<V: Ve<T, D>, M: Base, const D: usize>(
        &self, 
        get_value_data: GetValueData,
        collapser: &Collapser,
        mat: M,
        tree: &mut CSGTree<M, V, T, D>,
    ) -> (Vec<usize>, bool) {

        match &self {
            VolumeValue::Sphere { pos, size } => {
    
                let (pos, r_0) = collapser.template.get_position_value::<V, D>(*pos)
                    .get_value(get_value_data, collapser);


                let (size, r_1) = collapser.template.get_number_value(*size)
                    .get_value(get_value_data, collapser);

                let mut roots = vec![];
                for (pos, size) in iproduct!(pos, size) {
                    roots.push(tree.add_sphere(pos.to_vecf(), size.to_f32(), mat));
                }

                (roots, r_0 || r_1)            
            },
            VolumeValue::Disk { pos, size, height } => {
   
                let (pos, r_0) = collapser.template.get_position_value::<V, D>(*pos)
                    .get_value(get_value_data, collapser);

                let (size, r_1) = collapser.template.get_number_value(*size)
                    .get_value(get_value_data, collapser);

                let (height, r_2) = collapser.template.get_number_value(*height)
                    .get_value(get_value_data, collapser);

                let mut roots = vec![];
                for (pos, size, height) in iproduct!(pos, size, height) {
                    let tree: &mut CSGTree<M, V3, T, 3> = unsafe { TreeUnion { a: tree }.b };
                    let pos: V3 = unsafe { PositionUnion { a: pos }.b };

                    roots.push(tree.add_disk(pos.to_vecf(), size.to_f32(), height.to_f32(), mat));
                }

                (roots, r_0 || r_1 || r_2)            
            },
            VolumeValue::Box { pos, size } => {
                let (pos, r_0) = collapser.template.get_position_value::<V, D>(*pos)
                    .get_value(get_value_data, collapser);

                let (size, r_1) = collapser.template.get_position_value::<V, D>(*size)
                    .get_value(get_value_data, collapser);


                let mut roots = vec![];
                for (pos, size) in iproduct!(pos, size) {
                    roots.push(tree.add_box(pos.to_vecf(), size.to_vecf(), mat));
                }

                (roots, r_0 || r_1)
            },
            VolumeValue::Union { a, b } => {
                let (mut a, r_0) = collapser.template.get_volume_value(*a)
                    .get_value_inner(get_value_data, collapser, mat, tree);
          
                let (mut b, r_1) = collapser.template.get_volume_value(*b)
                    .get_value_inner(get_value_data, collapser, mat, tree);

                a.append(&mut b);

                (a, r_0 || r_1)
            },
            VolumeValue::Cut { base, cut } => {
                let (mut base, r_0) = collapser.template.get_volume_value(*base)
                    .get_value_inner(get_value_data, collapser, mat, tree);
          
                let (mut cut, r_1) = collapser.template.get_volume_value(*cut)
                    .get_value_inner(get_value_data, collapser, mat, tree);

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
            VolumeValue::Material { mat: new_mat, child } => {

                // Compile time if statement: 
                // If M is u8 then the child will be created with new_mat as material
                // otherwise just create the child
                if TypeId::of::<M>() == TypeId::of::<u8>() {
                    
                    let tree: &mut CSGTree<u8, V, T, D> = unsafe { TreeUnion { a: tree }.b };

                    collapser.template.get_volume_value(*child)
                        .get_value_inner(get_value_data, collapser, *new_mat, tree)
                } else {
                    collapser.template.get_volume_value(*child)
                        .get_value_inner(get_value_data, collapser, mat, tree) 
                }
            },
        }
    }
}
