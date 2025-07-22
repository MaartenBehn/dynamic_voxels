use std::usize;

use octa_force::{glam::{Mat4, Vec3}, puffin_egui::puffin};


use crate::{csg::vec_csg_tree::tree::CSG_PARENT_NONE, util::aabb3d::AABB};

use super::tree::{VecCSGNode, VecCSGNodeData, VecCSGTree};

impl<T: Clone> VecCSGTree<T> {
    
    pub fn set_all_aabbs(&mut self) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        let mut propergate_ids = vec![];
        self.set_primitive_aabbs(0, &Mat4::IDENTITY, &mut propergate_ids);
        
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

    pub fn set_primitive_aabbs(&mut self, i: usize, base_mat: &Mat4, changed_nodes: &mut Vec<usize>) {
        let node = &self.nodes[i];
        match &node.data {
            VecCSGNodeData::Union(c1, c2)
            | VecCSGNodeData::Remove(c1, c2) 
            | VecCSGNodeData::Intersect(c1, c2) => {
                let (c1, c2) = (*c1, *c2); // To please borrow checker
                self.set_primitive_aabbs(c1, base_mat, changed_nodes);
                self.set_primitive_aabbs(c2, base_mat, changed_nodes);
            },
            VecCSGNodeData::Mat(mat, c1) => {
                let mat = mat.mul_mat4(base_mat);
                self.set_primitive_aabbs(*c1, &mat, changed_nodes);
            },
            VecCSGNodeData::Box(mat, ..) => {
                let mat = mat.mul_mat4(base_mat);
                self.nodes[i].aabb = AABB::from_box(&mat);
                changed_nodes.push(i);
            },
            VecCSGNodeData::Sphere(mat, ..) => {
                let mat = mat.mul_mat4(base_mat);
                self.nodes[i].aabb = AABB::from_sphere(&mat);
                changed_nodes.push(i);
            },
            VecCSGNodeData::VoxelGrid(grid, pos) => {
                self.nodes[i].aabb = AABB::new(
                    (grid.size / 2).as_vec3() * -1.0 + pos.as_vec3(),
                    (grid.size / 2).as_vec3() + pos.as_vec3());

                changed_nodes.push(i);
            }
            VecCSGNodeData::All(_) => {
                self.nodes[i].aabb = AABB::infinte();
                changed_nodes.push(i);
            },
        }
    }

    pub fn propergate_aabb_change(&mut self, i: usize) -> usize {
        let node = self.nodes[i].to_owned();
        match node.data {
            VecCSGNodeData::Union(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb.union(self.nodes[c2].aabb);
            }
            VecCSGNodeData::Remove(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb;
            }
            VecCSGNodeData::Intersect(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb.intersect(self.nodes[c2].aabb);
            }
            VecCSGNodeData::Mat(_, c1) => {
                self.nodes[i].aabb = self.nodes[c1].aabb;
            }
            _ => {
                panic!("propergate_aabb_change can only be called for Union, Remove or Intersect")
            }
        }

        node.parent
    }
}

