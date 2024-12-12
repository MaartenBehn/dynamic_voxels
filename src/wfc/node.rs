use crate::{
    aabb::AABB,
    cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE},
};
use ::octa_force::glam::Mat4;
use octa_force::glam::{ivec3, vec3, vec4, Vec3, Vec4Swizzles};

use super::controller::WFCController;

#[derive(Debug, Clone)]
pub enum WFCNode {
    None,
    NumberSet(Vec<usize>),
    Number(usize),
    Volume(CSGTree),
    Pos(Vec3),

    Box {
        mat: Mat4,
        max_pipe_nodes: usize,
        min_pipe_nodes: usize,

        num_pipe_node: usize,
    },
    NumPipeNodes {
        box_index: usize,
        num: Vec<usize>,
    },
    PipeNode {
        box_index: usize,
        nr: usize,
        pipe_connect_1: Vec<usize>,
        pipe_connect_2: Vec<usize>,
        pos_index: usize,
        valid: bool,
    },
}

impl WFCNode {
    pub fn new_pipe_node(index: usize, box_index: usize) -> WFCNode {
        WFCNode::PipeNode {
            box_index,
            nr: index,
            pipe_connect_1: vec![],
            pipe_connect_2: vec![],
            pos_index: 0,
            valid: true,
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
                self.add_node(WFCNode::NumPipeNodes {
                    box_index: index,
                    num: (*min_pipe_nodes..=*max_pipe_nodes).collect(),
                });
            }
            WFCNode::NumPipeNodes { box_index, num } => {
                let max = *num.last().unwrap_or(&0);

                let box_index = *box_index;
                for i in 0..max {
                    self.add_node(WFCNode::new_pipe_node(i, box_index));
                }
            }
            WFCNode::PipeNode { box_index, .. } => {
                let box_node = &self.nodes[*box_index];
                assert!(
                    matches!(box_node, WFCNode::Box { .. }),
                    "PipeNode box_index needs to be a index of a Box"
                );

                let mat = match box_node {
                    WFCNode::Box { mat, .. } => mat,
                    _ => unreachable!(),
                };

                let mut csg = CSGTree::new();
                csg.nodes = vec![CSGNode::new(CSGNodeData::Box(*mat, MATERIAL_NONE))];
                csg.set_all_aabbs();

                let pos_index = self.add_node(WFCNode::Volume(csg));

                match &mut self.nodes[index] {
                    WFCNode::PipeNode { pos_index: pos, .. } => *pos = pos_index,
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
    }

    pub fn collapse(&mut self, index: usize) {
        let node = &self.nodes[index];

        match node {
            WFCNode::Box { .. } => {}
            WFCNode::NumPipeNodes { .. } => {}
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
                    match &mut self.nodes[index] {
                        WFCNode::PipeNode { valid, .. } => *valid = false,
                        _ => unreachable!(),
                    }

                    return;
                }

                self.nodes[pos_index] = WFCNode::None;

                let pos_index = self.add_node(WFCNode::Pos(new_pos.unwrap()));

                match &mut self.nodes[index] {
                    WFCNode::PipeNode { pos_index: pos, .. } => *pos = pos_index,
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
    }
}

impl CSGTree {
    pub fn find_valid_pos(&self, grid_size: f32) -> Option<Vec3> {
        let aabb = &self.nodes[0].aabb;

        let min = (aabb.min * grid_size).as_ivec3();
        let max = (aabb.max * grid_size).as_ivec3();
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..max.z {
                    let pos = ivec3(x, y, z).as_vec3() / grid_size;
                    if self.at_pos(pos) {
                        return Some(pos);
                    }
                }
            }
        }

        None
    }

    pub fn at_pos(&self, pos: Vec3) -> bool {
        self.at_pos_internal(pos, 0)
    }

    fn at_pos_internal(&self, pos: Vec3, index: usize) -> bool {
        let node = &self.nodes[index];

        match node.data {
            CSGNodeData::Union(c1, c2) => {
                self.at_pos_internal(pos, c1) || self.at_pos_internal(pos, c2)
            }
            CSGNodeData::Remove(c1, c2) => {
                self.at_pos_internal(pos, c1) && !self.at_pos_internal(pos, c2)
            }
            CSGNodeData::Intersect(c1, c2) => {
                self.at_pos_internal(pos, c1) && self.at_pos_internal(pos, c2)
            }
            CSGNodeData::Box(mat, _) => {
                let pos = mat.mul_vec4(vec4(pos.x, pos.y, pos.z, 1.0)).xyz();

                let aabb = AABB {
                    min: vec3(-0.5, -0.5, -0.5),
                    max: vec3(0.5, 0.5, 0.5),
                };

                aabb.pos_in_aabb(pos)
            }
            CSGNodeData::Sphere(mat, _) => {
                let pos = mat.mul_vec4(vec4(pos.x, pos.y, pos.z, 1.0)).xyz();

                pos_in_sphere(pos, vec3(0.0, 0.0, 0.0), 1.0)
            }
            CSGNodeData::VoxelVolume(_) => todo!(),
        }
    }
}

fn pos_in_sphere(pos: Vec3, s_pos: Vec3, radius: f32) -> bool {
    pos.distance(s_pos) < radius
}
