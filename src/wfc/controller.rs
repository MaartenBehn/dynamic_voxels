use octa_force::glam::Mat4;

use super::node::WFCNode;

#[derive(Clone, Debug)]
pub struct WFCController {
    pub nodes: Vec<WFCNode>,
}

impl WFCController {
    pub fn new() -> Self {
        WFCController { nodes: vec![] }
    }

    pub fn set_example(&mut self) {
        self.add_node(WFCNode::Box {
            mat: Mat4::default(),
            max_pipe_nodes: 5,
            min_pipe_nodes: 1,
            num_pipe_node: 0,
        });
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
