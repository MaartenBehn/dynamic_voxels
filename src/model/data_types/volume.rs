use egui_snarl::OutPinId;
use itertools::Itertools;
use smallvec::SmallVec;

use crate::{csg::{csg_tree::tree::CSGTree, Base}, model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::ComposeNodeType, ModelComposer}, template::{update::MakeTemplateData, value::ComposeTemplateValue}}, util::{iter_merger::IM5, number::Nu, vector::Ve}};

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
    pub fn make_volume<V: Ve<T, D>, const D: usize>(
        &self, 
        pin: OutPinId,
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> ValueIndexVolume {
        let value_index = pin.node.0;
        if data.template.has_value(value_index) {
            return value_index;   
        } 

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        let value = match &node.t {
            ComposeNodeType::Sphere => VolumeTemplate::Sphere { 
                pos: self.make_position(node, 0, data),
                size: self.make_number(node, 1, data),
            },
            ComposeNodeType::Box2D
            | ComposeNodeType::Box3D => VolumeTemplate::Box { 
                pos: self.make_position(node, 0, data),
                size: self.make_position(node, 1, data),
            },
            ComposeNodeType::UnionVolume2D
            | ComposeNodeType::UnionVolume3D => VolumeTemplate::Union {
                a: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                b: self.make_volume(self.get_input_remote_pin_by_index(node, 1), data),
            },
            ComposeNodeType::CutVolume2D
            | ComposeNodeType::CutVolume3D => VolumeTemplate::Cut {
                base: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                cut: self.make_volume(self.get_input_remote_pin_by_index(node, 1), data),
            },
            ComposeNodeType::CircleUnion => VolumeTemplate::SphereUnion { 
                position_set: self.make_position_set(self.get_input_remote_pin_by_type(node, ComposeDataType::PositionSet2D), data), 
                size: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data) 
            },
            ComposeNodeType::SphereUnion => VolumeTemplate::SphereUnion { 
                position_set: self.make_position_set(self.get_input_remote_pin_by_type(node, ComposeDataType::PositionSet3D), data), 
                size: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data) 
            },
            _ => unreachable!(),
        };

        data.template.set_value(value_index, ComposeTemplateValue::Volume(value));
        return value_index;
    }
}

impl<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> VolumeTemplate<V, V2, V3, T, D> { 
    pub fn get_value<B: BS<V2, V3, T>, M: Base>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        mat: M, 
    ) -> (CSGTree<M, V, T, D>, bool)  {
        self.get_value_inner(self.root, get_value_data, collapser, mat)
    }
 
    pub fn get_value_inner<B: BS<V2, V3, T>, M: Base>(
        &self, 
        index: usize, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        mat: M,
    ) -> (CSGTree<M, V, T, D>, bool) {

        let node = &self.nodes[index];
        match &node {
            VolumeTemplateData::Sphere { pos, size } => {
                let (pos, r_0) = pos.get_value(get_value_data, collapser);
                let (size, r_1) = size.get_value(get_value_data, collapser);

                let csg = pos.into_iter().cartesian_product(size)
                    .map(|(pos, size)| {
                        CSGTree::new_sphere(
                            pos, 
                            size, 
                            mat)
                    })
                    .fold(CSGTree::default(), |mut a, b| {
                        a.union_at_root(&b.nodes, b.root);
                        a
                    });

                (csg, r_0 || r_1)
            },
            VolumeTemplateData::Box { pos, size } => {
                let (pos, r_0) = pos.get_value(get_value_data, collapser);
                let (size, r_1) = size.get_value(get_value_data, collapser);
                
                let csg = pos.into_iter().cartesian_product(size)
                    .map(|(pos, size)| {
                        CSGTree::new_box(
                            pos, 
                            size, 
                            mat)
                    })
                    .fold(CSGTree::default(), |mut a, b| {
                        a.union_at_root(&b.nodes, b.root);
                        a
                    });                
                (csg, r_0 || r_1)
            },
            VolumeTemplateData::Union { a, b } => {
                let (mut a, r_0) = self.get_value_inner(*a, get_value_data, collapser, mat);
                let (b, r_1) = self.get_value_inner(*b, get_value_data, collapser, mat);

                a.union_at_root(&b.nodes, b.root);

                (a, r_0 || r_1)
            },
            VolumeTemplateData::Cut { base, cut } => {
                let (mut a, r_0) = self.get_value_inner(*base, get_value_data, collapser, mat);
                let (b, r_1) = self.get_value_inner(*cut, get_value_data, collapser, mat);
                
                a.cut_at_root(&b.nodes, b.root);

                (a, r_0 || r_1)
            },
            VolumeTemplateData::SphereUnion { position_set, size } => {

                let (radius, r_0) = size.get_value(get_value_data, collapser);
                let (set, r_1) = position_set.get_value::<V, B, D>(get_value_data, collapser);
                
                let mut csg = CSGTree::default();

                let radius = radius.into_iter().map(|r| r.to_f32()).collect::<SmallVec<[_; 1]>>();
                for pos in set {
                    let pos = pos.to_vecf();

                    for radius in radius.iter() {
                        csg.union_sphere(pos, *radius, mat);
                    }
                }
               
                (csg, r_0 || r_1)
            },
        }
    }

    pub fn cut_loop(&mut self, to_index: usize) {
        self.cut_loop_inner(self.root, to_index);
    }

    pub fn cut_loop_inner(&mut self, i: usize, to_index: usize) {
        let node: &mut VolumeTemplateData<V, V2, V3, T, D> = &mut self.nodes[i];
        match node {
            VolumeTemplateData::Sphere { pos, size } => {
                pos.cut_loop(to_index);
            },
            VolumeTemplateData::Box { pos, size } => {
                pos.cut_loop(to_index);
                size.cut_loop(to_index);
            },
            VolumeTemplateData::Union { a, b } => {
                let a = *a;
                let b = *b;

                self.cut_loop_inner(a, to_index);
                self.cut_loop_inner(b, to_index);
            },
            VolumeTemplateData::Cut { base, cut } => {
                let base = *base;
                let cut = *cut;

                self.cut_loop_inner(base, to_index);
                self.cut_loop_inner(cut, to_index);
            },
            VolumeTemplateData::SphereUnion { position_set, size } => {
                position_set.cut_loop(to_index);
            },
        }
    }
}
