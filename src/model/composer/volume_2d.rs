use egui_snarl::OutPinId;

use super::{data_type::ComposeDataType, nodes::ComposeNodeType, pos_space::PositionSpace, primitive::{Number, Position2D, Position3D, PositionSet}, ModelComposer};

#[derive(Debug, Clone, Copy)]
pub enum Volume2DData {
    Circle {
        pos: Position2D,
        size: Number
    },
    Box {
        pos: Position2D,
        size: Position2D,
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
pub struct Volume2D {
    pub nodes: Vec<Volume2DData>,
    pub root: usize,
}

impl ModelComposer {
    pub fn make_volume_2d(&self, pin: OutPinId) -> Volume2D {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::Circle => Volume2D {
                nodes: vec![Volume2DData::Circle { 
                    pos: self.make_position2d(node, self.get_input_index_by_type(node, ComposeDataType::Position2D(None))),
                    size: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None))) 
                }],
                root: 0,
            },
            ComposeNodeType::Box => Volume2D {
                nodes: vec![Volume2DData::Box { 
                    pos: self.make_position2d(node, 0),
                    size: self.make_position2d(node, 1),
                }],
                root: 0,
            },
            ComposeNodeType::UnionVolume2D => {
                let mut a = self.make_volume_2d(self.get_input_node_by_index(node, 0));
                let mut b = self.make_volume_2d(self.get_input_node_by_index(node, 1));

                let mut nodes = vec![];

                let a_root = a.root;
                nodes.append(&mut a.nodes);

                let b_root = b.root + nodes.len();
                nodes.append(&mut b.nodes);

                let root = nodes.len();
                nodes.push(Volume2DData::Union { a: a_root, b: b_root });

                Volume2D {
                    nodes,
                    root,
                }
            },
            ComposeNodeType::CutVolume2D => {
                let mut base = self.make_volume_2d(self.get_input_node_by_index(node, 0));
                let mut cut = self.make_volume_2d(self.get_input_node_by_index(node, 1));

                let mut nodes = vec![];

                let base_root = base.root;
                nodes.append(&mut base.nodes);

                let cut_root = cut.root + nodes.len();
                nodes.append(&mut cut.nodes);

                let root = nodes.len();
                nodes.push(Volume2DData::Cut { base: base_root, cut: cut_root });

                Volume2D {
                    nodes,
                    root,
                }
            },
            ComposeNodeType::CircleUnion => Volume2D {
                nodes: vec![Volume2DData::SphereUnion { 
                    position_set: self.make_position_set(self.get_input_node_by_type(node, ComposeDataType::PositionSet)), 
                    size: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None))) 
                }],
                root: 0,
            },
            _ => unreachable!(),
        }
    }
}
