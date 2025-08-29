use egui_snarl::OutPinId;
use itertools::Itertools;

use super::{data_type::ComposeDataType, nodes::ComposeNodeType, pos_space::PositionSpace, primitive::{Number, Position3D, PositionSet}, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

#[derive(Debug, Clone, Copy)]
pub enum Volume3DData {
    Sphere {
        pos: Position3D,
        size: Number
    },
    Box {
        pos: Position3D,
        size: Position3D,
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
        position_set: PositionSet,
        size: Number,
    },
}

#[derive(Debug, Clone)]
pub struct Volume3D {
    pub nodes: Vec<Volume3DData>,
    pub root: usize,
}

impl ModelComposer {
    pub fn make_volume_3d(&self, pin: OutPinId, template: &ComposeTemplate) -> Volume3D {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::Sphere => Volume3D {
                nodes: vec![Volume3DData::Sphere { 
                    pos: self.make_position3d(node, self.get_input_index_by_type(node, ComposeDataType::Position3D(None)), template),
                    size: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template) 
                }],
                root: 0,
            },
            ComposeNodeType::Box => Volume3D {
                nodes: vec![Volume3DData::Box { 
                    pos: self.make_position3d(node, 0, template),
                    size: self.make_position3d(node, 1, template),
                }],
                root: 0,
            },
            ComposeNodeType::UnionVolume3D => {
                let mut a = self.make_volume_3d(self.get_input_node_by_index(node, 0), template);
                let mut b = self.make_volume_3d(self.get_input_node_by_index(node, 1), template);

                let mut nodes = vec![];

                let a_root = a.root;
                nodes.append(&mut a.nodes);

                let b_root = b.root + nodes.len();
                nodes.append(&mut b.nodes);

                let root = nodes.len();
                nodes.push(Volume3DData::Union { a: a_root, b: b_root });

                Volume3D {
                    nodes,
                    root,
                }
            },
            ComposeNodeType::CutVolume3D => {
                let mut base = self.make_volume_3d(self.get_input_node_by_index(node, 0), template);
                let mut cut = self.make_volume_3d(self.get_input_node_by_index(node, 1), template);

                let mut nodes = vec![];

                let base_root = base.root;
                nodes.append(&mut base.nodes);

                let cut_root = cut.root + nodes.len();
                nodes.append(&mut cut.nodes);

                let root = nodes.len();
                nodes.push(Volume3DData::Cut { base: base_root, cut: cut_root });

                Volume3D {
                    nodes,
                    root,
                }
            },
            ComposeNodeType::SphereUnion => Volume3D {
                nodes: vec![Volume3DData::SphereUnion { 
                    position_set: self.make_position_set(self.get_input_node_by_type(node, ComposeDataType::PositionSet), template), 
                    size: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template) 
                }],
                root: 0,
            },
            _ => unreachable!(),
        }
    }
}

impl Volume3D {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        self.get_dependend_template_nodes_inner(self.root)
    }

    fn get_dependend_template_nodes_inner(&self, index: usize) -> impl Iterator<Item = TemplateIndex> {
        let node = &self.nodes[index];
        match node {
            Volume3DData::Sphere { pos, size } => {
                pos.get_dependend_template_nodes()
                    .chain(size.get_dependend_template_nodes()).collect_vec()
            },
            Volume3DData::Box { pos, size } => {
                pos.get_dependend_template_nodes()
                    .chain(size.get_dependend_template_nodes()).collect_vec()
            },
            Volume3DData::Union { a, b } => {
                self.get_dependend_template_nodes_inner(*a)
                    .chain(self.get_dependend_template_nodes_inner(*b))
                    .collect_vec()
            },
            Volume3DData::Cut { base, cut } => {
                self.get_dependend_template_nodes_inner(*base)
                    .chain(self.get_dependend_template_nodes_inner(*cut))
                    .collect_vec()
            },
            Volume3DData::SphereUnion { position_set, size } => {
                position_set.get_dependend_template_nodes()
                    .chain(size.get_dependend_template_nodes())
                    .collect_vec()
            },
        }.into_iter()
    }
}
