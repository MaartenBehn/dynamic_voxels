use std::usize;

use octa_force::{glam::{Mat4, Vec3}, puffin_egui::puffin};

use crate::{aabb::AABB, csg_tree::tree::{CSGNodeData, CSG_PARENT_NONE}};

use super::tree::{CSGNode, CSGTree};

impl CSGTree {
    
    pub fn set_all_aabbs(&mut self, padding: f32) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        let mut propergate_ids = vec![];
        self.set_primitive_aabbs(0, &Mat4::IDENTITY, &mut propergate_ids, padding);
        
        propergate_ids = self.get_id_parents(&propergate_ids);
        while !propergate_ids.is_empty() {
            let mut new_ids = vec![];
            for id in propergate_ids.iter() {
                let parent = self.propergate_aabb_change(*id);
                if parent != CSG_PARENT_NONE {
                    new_ids.push(parent);
                }
            }

            propergate_ids = new_ids;
        }
    }

    pub fn set_primitive_aabbs(&mut self, i: usize, base_mat: &Mat4, changed_nodes: &mut Vec<usize>, padding: f32) {
        let node = &self.nodes[i];
        match &node.data {
            CSGNodeData::Union(c1, c2)
            | CSGNodeData::Remove(c1, c2) 
            | CSGNodeData::Intersect(c1, c2) => {
                let (c1, c2) = (*c1, *c2); // To please borrow checker
                self.set_primitive_aabbs(c1, base_mat, changed_nodes, padding);
                self.set_primitive_aabbs(c2, base_mat, changed_nodes, padding);
            },
            CSGNodeData::Mat(mat, c1) => {
                let mat = mat.mul_mat4(base_mat);
                self.set_primitive_aabbs(*c1, &mat, changed_nodes, padding);
            },
            CSGNodeData::Box(mat, ..) => {
                let mat = mat.mul_mat4(base_mat);
                self.nodes[i].aabb = AABB::from_box(&mat, padding);
                changed_nodes.push(i);
            },
            CSGNodeData::Sphere(mat, ..) => {
                let mat = mat.mul_mat4(base_mat);
                self.nodes[i].aabb = AABB::from_sphere(&mat, padding);
                changed_nodes.push(i);
            },
            CSGNodeData::VoxelGrid(grid, pos) => {
                self.nodes[i].aabb = AABB{
                    min: (grid.size / 2).as_vec3() * -1.0 + pos.as_vec3(),
                    max: (grid.size / 2).as_vec3() + pos.as_vec3(),
                };
                changed_nodes.push(i);
            }
            CSGNodeData::All(_) => {
                self.nodes[i].aabb = AABB{
                    min: Vec3::NEG_INFINITY, 
                    max: Vec3::INFINITY,
                };
                changed_nodes.push(i);
            },
        }
    }

    pub fn propergate_aabb_change(&mut self, i: usize) -> usize {
        let node = self.nodes[i].to_owned();
        match node.data {
            CSGNodeData::Union(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb.union(self.nodes[c2].aabb);
            }
            CSGNodeData::Remove(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb;
            }
            CSGNodeData::Intersect(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb.intersect(self.nodes[c2].aabb);
            }
            CSGNodeData::Mat(_, c1) => {
                self.nodes[i].aabb = self.nodes[c1].aabb;
            }
            _ => {
                panic!("propergate_aabb_change can only be called for Union, Remove or Intersect")
            }
        }

        node.parent
    }
}

