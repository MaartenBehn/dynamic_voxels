use std::usize;

use crate::{
    aabb::AABB,
    cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE, VOXEL_SIZE},
};
use octa_force::glam::{ivec3, vec3, vec4, Mat4, Quat, Vec3, Vec4Swizzles};

use super::controller::WFCController;

#[derive(Debug, Clone)]
pub enum WFCNode {
    None,
    NumberSet(Vec<usize>),
    Number(usize),
    Volum {
        csg: CSGTree,
        children: Vec<usize>,
    },
    Pos(Vec3),

    Box {
        pipe_volume_index: usize,
        max_pipe_nodes: usize,
        min_pipe_nodes: usize,

        num_pipe_node_index: usize,
    },
    NumPipeNodes {
        box_index: usize,
        num: Vec<usize>,
        pipe_node_indecies: Vec<usize>,
    },
    PipeNode {
        box_index: usize,
        nr: usize,
        pipe_connect_1_index: Vec<usize>,
        pipe_connect_2_index: Vec<usize>,
        pos_index: usize,
    },
    PipeConnection {},
}

impl WFCNode {
    pub fn new_pipe_node(index: usize, box_index: usize) -> WFCNode {
        WFCNode::PipeNode {
            box_index,
            nr: index,
            pipe_connect_1_index: vec![],
            pipe_connect_2_index: vec![],
            pos_index: 0,
        }
    }
}

impl WFCController {
    pub fn add_depends(&mut self, index: usize) {
        let node = &self.nodes[index];

        match node {
            WFCNode::Box {
                max_pipe_nodes,
                min_pipe_nodes,
                ..
            } => {
                let i = self.add_node(WFCNode::NumPipeNodes {
                    box_index: index,
                    num: (*min_pipe_nodes..=*max_pipe_nodes).collect(),
                    pipe_node_indecies: vec![],
                });

                match &mut self.nodes[index] {
                    WFCNode::Box {
                        num_pipe_node_index,
                        ..
                    } => {
                        *num_pipe_node_index = i;
                    }
                    _ => unreachable!(),
                }
            }
            WFCNode::NumPipeNodes { box_index, num, .. } => {
                let max = *num.last().unwrap_or(&0);

                let box_index = *box_index;
                for i in 0..max {
                    let pipe_index = self.add_node(WFCNode::new_pipe_node(i, box_index));
                    match &mut self.nodes[index] {
                        WFCNode::NumPipeNodes {
                            pipe_node_indecies, ..
                        } => {
                            pipe_node_indecies.push(pipe_index);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            WFCNode::PipeNode { box_index, .. } => {
                let box_node = &self.nodes[*box_index];
                assert!(
                    matches!(box_node, WFCNode::Box { .. }),
                    "PipeNode box_index needs to be a index of a Box"
                );

                let volume_index = match box_node {
                    WFCNode::Box {
                        pipe_volume_index: pipe_volume,
                        ..
                    } => *pipe_volume,
                    _ => unreachable!(),
                };

                match &mut self.nodes[index] {
                    WFCNode::PipeNode { pos_index: pos, .. } => *pos = volume_index,
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
    }

    pub fn collapse(&mut self, index: usize) -> bool {
        let node = &self.nodes[index];

        match node {
            WFCNode::Box {
                num_pipe_node_index,
                ..
            } => {
                self.collapse(*num_pipe_node_index);

                true
            }

            WFCNode::NumPipeNodes {
                pipe_node_indecies,
                num,
                ..
            } => {
                let min = num.first().copied();
                let mut valid_indexies = vec![];

                for pipe_node_index in pipe_node_indecies.to_owned() {
                    let valid = self.collapse(pipe_node_index);

                    if valid {
                        valid_indexies.push(pipe_node_index);
                    }
                }

                if min.is_none() || valid_indexies.len() < min.unwrap() {
                    return false;
                }

                match &mut self.nodes[index] {
                    WFCNode::NumPipeNodes {
                        num,
                        pipe_node_indecies,
                        ..
                    } => {
                        *num = vec![valid_indexies.len()];
                        *pipe_node_indecies = valid_indexies;
                    }
                    _ => unreachable!(),
                }

                true
            }
            WFCNode::PipeNode { pos_index, .. } => {
                let pos_index = *pos_index;
                let pos_node = &self.nodes[pos_index];
                assert!(
                    matches!(pos_node, WFCNode::Volume { .. }),
                    "PipeNode pos needs to be a index of a Volume"
                );

                let new_pos = match pos_node {
                    WFCNode::Volume(csg) => csg.find_valid_pos(0.1),
                    _ => unreachable!(),
                };

                if new_pos.is_none() {
                    return false;
                }
                let new_pos = new_pos.unwrap();

                match &mut self.nodes[pos_index] {
                    WFCNode::Volume(csg) => {
                        let mut tree = CSGTree::new();
                        tree.nodes.push(CSGNode::new(CSGNodeData::Sphere(
                            Mat4::from_scale_rotation_translation(
                                Vec3::ONE * 0.1,
                                Quat::from_euler(octa_force::glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                                new_pos,
                            ),
                            MATERIAL_NONE,
                        )));
                        csg.append_tree_with_remove(tree);
                        csg.set_all_aabbs(0.0);
                    }
                    _ => unreachable!(),
                }

                let pos_index = self.add_node(WFCNode::Pos(new_pos));

                match &mut self.nodes[index] {
                    WFCNode::PipeNode { pos_index: pos, .. } => *pos = pos_index,
                    _ => unreachable!(),
                }

                true
            }
            _ => true,
        }
    }

    pub fn make_cgs(&self, index: usize) -> CSGTree {
        let node = &self.nodes[index];

        match node {
            WFCNode::Box {
                num_pipe_node_index,
                ..
            } => {
                let pipe_nodes_tree = self.make_cgs(*num_pipe_node_index);

                pipe_nodes_tree
            }
            WFCNode::NumPipeNodes {
                pipe_node_indecies, ..
            } => {
                let mut tree = None;
                for pipe_node_index in pipe_node_indecies {
                    let sub_tree = self.make_cgs(*pipe_node_index);

                    if tree.is_none() {
                        tree = Some(sub_tree);
                    } else {
                        tree.as_mut().unwrap().append_tree_with_union(sub_tree);
                    }
                }

                tree.unwrap()
            }
            WFCNode::PipeNode { pos_index, .. } => {
                let pos_node = &self.nodes[*pos_index];

                let pos = match pos_node {
                    WFCNode::Pos(pos) => *pos,
                    _ => unreachable!(),
                };

                let mut tree = CSGTree::new();
                tree.nodes.push(CSGNode::new(CSGNodeData::Sphere(
                    Mat4::from_scale_rotation_translation(
                        Vec3::ONE * 0.01 * VOXEL_SIZE,
                        Quat::from_euler(octa_force::glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                        pos * VOXEL_SIZE,
                    ),
                    1,
                )));

                tree
            }
            _ => unreachable!(),
        }
    }
}
