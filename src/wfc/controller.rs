use octa_force::glam::Mat4;

use crate::cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE};

use super::node::WFCNode;

#[derive(Clone, Debug)]
pub struct WFCController {
    pub nodes: Vec<WFCNode>,
    pub root_index: usize,
}

impl WFCController {
    pub fn new() -> Self {
        WFCController {
            nodes: vec![],
            root_index: 0,
        }
    }

    pub fn set_example(&mut self) {
        let mut csg = CSGTree::new();
        csg.nodes = vec![CSGNode::new(CSGNodeData::Box(
            Mat4::default(),
            MATERIAL_NONE,
        ))];
        csg.set_all_aabbs(0.0);

        csg.nodes[0].aabb.get_random_sampled_positions(0.1);

        self.add_node(WFCNode::Volume(csg));
        self.add_node(WFCNode::Box {
            max_pipe_nodes: 5,
            min_pipe_nodes: 1,
            num_pipe_node_index: 0,
            pipe_volume_index: 0,
        });

        self.root_index = 1;
    }

    pub fn add_node(&mut self, node: WFCNode) -> usize {
        let index = self.nodes.len();

        self.nodes.push(node);

        self.add_depends(index);

        index
    }
}

impl Default for WFCController {
    fn default() -> Self {
        Self::new()
    }
}
