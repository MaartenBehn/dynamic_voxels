use octa_force::glam::{vec3, vec4, Vec3, Vec4Swizzles};


use crate::util::aabb::AABB;

use super::tree::{VecCSGNode, VecCSGNodeData, VecCSGTree};

impl VecCSGTree {
    pub(super) fn find_valid_pos(&self, grid_size: f32) -> Option<Vec3> {
        let aabb = &self.nodes[0].aabb;

        aabb.get_random_sampled_positions(grid_size)
            .into_iter()
            .find(|&pos| self.at_pos(pos))
    }

    pub(super) fn at_pos(&self, pos: Vec3) -> bool {
        self.at_pos_internal(pos, 0)
    }

    fn at_pos_internal(&self, pos: Vec3, index: usize) -> bool {
        let node = &self.nodes[index];

        match node.data { 
            VecCSGNodeData::Union(c1, c2) => {
                self.at_pos_internal(pos, c1) || self.at_pos_internal(pos, c2)
            }
            VecCSGNodeData::Remove(c1, c2) => {
                self.at_pos_internal(pos, c1) && !self.at_pos_internal(pos, c2)
            }
            VecCSGNodeData::Intersect(c1, c2) => {
                self.at_pos_internal(pos, c1) && self.at_pos_internal(pos, c2)
            }
            VecCSGNodeData::Mat(_, c) => self.at_pos_internal(pos, c),
            VecCSGNodeData::Box(mat, _) => {
                let pos = mat.inverse().mul_vec4(vec4(pos.x, pos.y, pos.z, 1.0)).xyz();

                let aabb = AABB {
                    min: vec3(-0.5, -0.5, -0.5),
                    max: vec3(0.5, 0.5, 0.5),
                };

                aabb.pos_in_aabb(pos)
            }
            VecCSGNodeData::Sphere(mat, _) => {
                let pos = mat.inverse().mul_vec4(vec4(pos.x, pos.y, pos.z, 1.0)).xyz();

                pos_in_sphere(pos, vec3(0.0, 0.0, 0.0), 1.0)
            }
            VecCSGNodeData::VoxelGrid(..) => todo!(),
            VecCSGNodeData::All(_) => true,
        }
    }

    pub fn append_tree_with_remove(&mut self, mut tree: VecCSGTree) {
        self.insert_node_before(VecCSGNode::new(VecCSGNodeData::Remove(1, self.nodes.len() + 1)));

        tree.shift_node_pointers(self.nodes.len());

        self.nodes.append(&mut tree.nodes);
    }

    pub fn append_tree_with_union(&mut self, mut tree: VecCSGTree) { 
        self.insert_node_before(VecCSGNode::new(VecCSGNodeData::Union(1, self.nodes.len() + 1)));

        tree.shift_node_pointers(self.nodes.len());

        self.nodes.append(&mut tree.nodes);
    }

    pub fn shift_node_pointers(&mut self, ammount: usize) {
        for i in 0..self.nodes.len() {
            match &mut self.nodes[i].data {
                VecCSGNodeData::Union(c1, c2)
                | VecCSGNodeData::Remove(c1, c2)
                | VecCSGNodeData::Intersect(c1, c2) => {
                    *c1 += ammount;
                    *c2 += ammount;
                }
                _ => {}
            }
        }
    }
    pub fn insert_node_before(&mut self, node: VecCSGNode) {
        self.shift_node_pointers(1);

        self.nodes.insert(0, node);
    }

    pub fn get_id_parents(&self, ids: &[usize]) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                match node.data {
                    VecCSGNodeData::Union(child1, child2)
                    | VecCSGNodeData::Remove(child1, child2)
                    | VecCSGNodeData::Intersect(child1, child2) => {
                        if ids.contains(&child1) || ids.contains(&child2) {
                            return Some(i);
                        }
                    }
                    _ => {}
                }

                None
            })
            .collect()
    }
}

fn pos_in_sphere(pos: Vec3, s_pos: Vec3, radius: f32) -> bool {
    pos.distance(s_pos) < radius
}
