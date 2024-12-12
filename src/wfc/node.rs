use crate::cgs_tree::tree::CSGTree;
use ::octa_force::glam::Mat4;

use super::controller::WFCController;

#[derive(Debug, Clone)]
pub enum WFCNode {
    Number(Vec<usize>),
    Pos3(CSGTree),

    Box {
        mat: Mat4,
        max_pipe_nodes: usize,
        min_pipe_nodes: usize,

        num_pipe_node: usize,
    },
    NumPipeNodes(Vec<usize>),
    PipeNode {
        index: usize,
    },
}

impl WFCController {
    pub fn add_depends(&mut self, index: usize) {
        let node = &self.nodes[index];

        match node {
            WFCNode::Box {
                mat: _,
                max_pipe_nodes,
                min_pipe_nodes,
                num_pipe_node: _,
            } => {
                self.add_node(WFCNode::NumPipeNodes(
                    (*min_pipe_nodes..*max_pipe_nodes).collect(),
                ));
            }
            WFCNode::NumPipeNodes(val) => {
                let max = *val.last().unwrap_or(&0);

                for i in 0..max {
                    self.add_node(WFCNode::PipeNode { index: i });
                }
            }
            _ => {}
        }
    }
}
