use egui_snarl::OutPinId;
use itertools::Itertools;
use smallvec::SmallVec;

use crate::{csg::{csg_tree::tree::CSGTree, Base}, model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::ComposeNodeType, template::{Ammount, ComposeTemplate, MakeTemplateData, TemplateIndex}, ModelComposer}}, util::{number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, number::NumberTemplate, position::PositionTemplate, position_set::PositionSetTemplate};

#[derive(Debug, Clone)]
pub enum VolumeTemplateData<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> {
    Sphere {
        pos: PositionTemplate<V, V2, V3, T, D>,
        size: NumberTemplate<V2, V3, T>
    },
    Box {
        pos: PositionTemplate<V, V2, V3, T, D>,
        size: PositionTemplate<V, V2, V3, T, D>,
    },
    Union {
        a: usize,
        b: usize,
    },
    Cut {
        base: usize,
        cut: usize,
    },
    SphereUnion {
        position_set: PositionSetTemplate<V2, V3, T>,
        size: NumberTemplate<V2, V3, T>,
    },
}

#[derive(Debug, Clone)]
pub struct VolumeTemplate<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> {
    pub nodes: Vec<VolumeTemplateData<V, V2, V3, T, D>>,
    pub root: usize,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_volume<V: Ve<T, D>, const D: usize>(
        &self, 
        pin: OutPinId,
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> VolumeTemplate<V, V2, V3, T, D> {

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::Sphere => VolumeTemplate {
                nodes: vec![VolumeTemplateData::Sphere { 
                    pos: self.make_position(node, self.get_input_pin_index_by_type(node, match D {
                        2 => ComposeDataType::Position2D(None), 
                        3 => ComposeDataType::Position3D(None),
                        _ => unreachable!(),
                    }), data),
                    size: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data) 
                }],
                root: 0,
            },
            ComposeNodeType::Box2D => VolumeTemplate {
                nodes: vec![VolumeTemplateData::Box { 
                    pos: self.make_position(node, 0, data),
                    size: self.make_position(node, 1, data),
                }],
                root: 0,
            },
            ComposeNodeType::Box3D => VolumeTemplate {
                nodes: vec![VolumeTemplateData::Box { 
                    pos: self.make_position(node, 0, data),
                    size: self.make_position(node, 1, data),
                }],
                root: 0,
            },
            ComposeNodeType::UnionVolume2D
            | ComposeNodeType::UnionVolume3D => {
                let mut a = self.make_volume(self.get_input_remote_pin_by_index(node, 0), data);
                let mut b = self.make_volume(self.get_input_remote_pin_by_index(node, 1), data);

                let mut nodes = vec![];
 
                let a_root = a.root + nodes.len();
                a.shift_ptrs(nodes.len());
                nodes.append(&mut a.nodes);

                let b_root = b.root + nodes.len();
                b.shift_ptrs(nodes.len());
                nodes.append(&mut b.nodes);

                let root = nodes.len();
                nodes.push(VolumeTemplateData::Union { a: a_root, b: b_root });

                VolumeTemplate {
                    nodes,
                    root,
                }
            },
            ComposeNodeType::CutVolume2D
            | ComposeNodeType::CutVolume3D => {
                let mut base = self.make_volume(self.get_input_remote_pin_by_index(node, 0), data);
                let mut cut = self.make_volume(self.get_input_remote_pin_by_index(node, 1), data);

                let mut nodes = vec![];

                let base_root = base.root + nodes.len();
                base.shift_ptrs(nodes.len());
                nodes.append(&mut base.nodes);

                let cut_root = cut.root + nodes.len();
                cut.shift_ptrs(nodes.len());
                nodes.append(&mut cut.nodes);

                let root = nodes.len();
                nodes.push(VolumeTemplateData::Cut { base: base_root, cut: cut_root });

                VolumeTemplate {
                    nodes,
                    root,
                }
            },
            ComposeNodeType::CircleUnion => VolumeTemplate {
                nodes: vec![VolumeTemplateData::SphereUnion { 
                    position_set: self.make_position_set(self.get_input_remote_pin_by_type(node, ComposeDataType::PositionSet2D), data), 
                    size: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data) 
                }],
                root: 0,
            },
            ComposeNodeType::SphereUnion => VolumeTemplate {
                nodes: vec![VolumeTemplateData::SphereUnion { 
                    position_set: self.make_position_set(self.get_input_remote_pin_by_type(node, ComposeDataType::PositionSet3D), data), 
                    size: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data) 
                }],
                root: 0,
            },
            _ => unreachable!(),
        }
    }
}

impl<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> VolumeTemplate<V, V2, V3, T, D> {
    fn shift_ptrs(&mut self, ammount: usize) {
        for node in &mut self.nodes {
            match node {
                VolumeTemplateData::Union { a, b } => {
                    *(a) += ammount;
                    *(b) += ammount;
                },
                VolumeTemplateData::Cut { base, cut } => {
                    *(base) += ammount;
                    *(cut) += ammount;
                },
                _ => {}
            }
        }
    }

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

                (CSGTree::new_sphere(
                pos, 
                size, 
                mat), r_0 || r_1)
            },
            VolumeTemplateData::Box { pos, size } => {
                let (pos, r_0) = pos.get_value(get_value_data, collapser);
                let (size, r_1) = size.get_value(get_value_data, collapser);

                (CSGTree::new_box(
                pos, 
                size, 
                mat), r_0 || r_1)
            },
            VolumeTemplateData::Union { a, b } => {
                let (mut a, r_0) = self.get_value_inner(*a, get_value_data, collapser, mat);
                let (b, r_1) = self.get_value_inner(*b, get_value_data, collapser, mat);
                a.union_at_root(&b.nodes, b.root);
                (a, r_0 || r_1)
            },
            VolumeTemplateData::Cut { base, cut } => {
                let (mut base, r_0) = self.get_value_inner(*base, get_value_data, collapser, mat);
                let (cut, r_1) = self.get_value_inner(*cut, get_value_data, collapser, mat);
                base.cut_at_root(&cut.nodes, cut.root);
                (base, r_0 || r_1)
            },
            VolumeTemplateData::SphereUnion { position_set, size } => {

                let (radius, r_0) = size.get_value(get_value_data, collapser);
                let radius = radius.to_f32();

                let mut csg = CSGTree::default();
                let (set, r_1) = position_set.get_value::<V, B, D>(get_value_data, collapser);

                for pos in set {
                    let pos = pos.to_vecf();
                    csg.union_sphere(pos, radius, mat);
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
