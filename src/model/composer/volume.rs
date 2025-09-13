use egui_snarl::OutPinId;
use itertools::Itertools;

use crate::{csg::{csg_tree::tree::CSGTree, Base}, util::{number::Nu, vector::Ve}};

use super::{build::BS, collapse::collapser::{CollapseNodeKey, Collapser}, data_type::ComposeDataType, nodes::ComposeNodeType, number::NumberTemplate, position::PositionTemplate, position_set::PositionSetTemplate, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

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
        position_set: PositionSetTemplate,
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
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> VolumeTemplate<V, V2, V3, T, D> {

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::Sphere => VolumeTemplate {
                nodes: vec![VolumeTemplateData::Sphere { 
                    pos: self.make_position(node, self.get_input_index_by_type(node, match D {
                        2 => ComposeDataType::Position2D(None), 
                        3 => ComposeDataType::Position3D(None),
                        _ => unreachable!(),
                    }), template),
                    size: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template) 
                }],
                root: 0,
            },
            ComposeNodeType::Box => VolumeTemplate {
                nodes: vec![VolumeTemplateData::Box { 
                    pos: self.make_position(node, 0, template),
                    size: self.make_position(node, 1, template),
                }],
                root: 0,
            },
            ComposeNodeType::UnionVolume3D => {
                let mut a = self.make_volume(self.get_input_pin_by_index(node, 0), template);
                let mut b = self.make_volume(self.get_input_pin_by_index(node, 1), template);

                let mut nodes = vec![];

                let a_root = a.root;
                nodes.append(&mut a.nodes);

                let b_root = b.root + nodes.len();
                nodes.append(&mut b.nodes);

                let root = nodes.len();
                nodes.push(VolumeTemplateData::Union { a: a_root, b: b_root });

                VolumeTemplate {
                    nodes,
                    root,
                }
            },
            ComposeNodeType::CutVolume3D => {
                let mut base = self.make_volume(self.get_input_pin_by_index(node, 0), template);
                let mut cut = self.make_volume(self.get_input_pin_by_index(node, 1), template);

                let mut nodes = vec![];

                let base_root = base.root;
                nodes.append(&mut base.nodes);

                let cut_root = cut.root + nodes.len();
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
                    position_set: self.make_position_set(self.get_input_pin_by_type(node, ComposeDataType::PositionSet2D), template), 
                    size: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template) 
                }],
                root: 0,
            },
            ComposeNodeType::SphereUnion => VolumeTemplate {
                nodes: vec![VolumeTemplateData::SphereUnion { 
                    position_set: self.make_position_set(self.get_input_pin_by_type(node, ComposeDataType::PositionSet3D), template), 
                    size: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template) 
                }],
                root: 0,
            },
            _ => unreachable!(),
        }
    }
}

impl<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> VolumeTemplate<V, V2, V3, T, D> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        self.get_dependend_template_nodes_inner(self.root)
    }

    fn get_dependend_template_nodes_inner(&self, index: usize) -> impl Iterator<Item = TemplateIndex> {
        let node = &self.nodes[index];
        match node {
            VolumeTemplateData::Sphere { pos, size } => {
                pos.get_dependend_template_nodes()
                    .chain(size.get_dependend_template_nodes()).collect_vec()
            },
            VolumeTemplateData::Box { pos, size } => {
                pos.get_dependend_template_nodes()
                    .chain(size.get_dependend_template_nodes()).collect_vec()
            },
            VolumeTemplateData::Union { a, b } => {
                self.get_dependend_template_nodes_inner(*a)
                    .chain(self.get_dependend_template_nodes_inner(*b))
                    .collect_vec()
            },
            VolumeTemplateData::Cut { base, cut } => {
                self.get_dependend_template_nodes_inner(*base)
                    .chain(self.get_dependend_template_nodes_inner(*cut))
                    .collect_vec()
            },
            VolumeTemplateData::SphereUnion { position_set, size } => {
                position_set.get_dependend_template_nodes()
                    .chain(size.get_dependend_template_nodes())
                    .collect_vec()
            },
        }.into_iter()
    }

    pub fn get_value<B: BS<V2, V3, T>, M: Base>(
        &self, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>,
        mat: M, 
    ) -> CSGTree<M, V, T, D>  {
        self.get_value_inner(self.root, depends, collapser, mat)
    }
 
    pub fn get_value_inner<B: BS<V2, V3, T>, M: Base>(
        &self, 
        index: usize, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>,
        mat: M,
    ) -> CSGTree<M, V, T, D> {

        let node = &self.nodes[index];
        match &node {
            VolumeTemplateData::Sphere { pos, size } => CSGTree::new_sphere(
                pos.get_value(depends, collapser), 
                size.get_value(depends, collapser), 
                mat
            ),
            VolumeTemplateData::Box { pos, size } => CSGTree::new_box(
                pos.get_value(depends, collapser), 
                size.get_value(depends, collapser), 
                mat
            ),
            VolumeTemplateData::Union { a, b } => {
                let mut a = self.get_value_inner(*a, depends, collapser, mat);
                let b = self.get_value_inner(*b, depends, collapser, mat);
                a.union_at_root(&b.nodes, b.root);
                a
            },
            VolumeTemplateData::Cut { base, cut } => {
                let mut base = self.get_value_inner(*base, depends, collapser, mat);
                let cut = self.get_value_inner(*cut, depends, collapser, mat);
                base.cut_at_root(&cut.nodes, cut.root);
                base
            },
            VolumeTemplateData::SphereUnion { position_set, size } => {

                let radius = size.get_value(depends, collapser).to_f32();
                let mut csg = CSGTree::default();
                for pos in position_set.get_value::<V, V2, V3, T, B, D>(depends, collapser) {
                    let pos = pos.to_vecf();
                    csg.union_sphere(pos, radius, mat);
                }

                csg
            },
        }
    }
}
