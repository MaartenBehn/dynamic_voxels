use octa_force::{glam::Mat4, puffin_egui::puffin};

use crate::{aabb::AABB, csg_tree::tree::CSGNodeData};

use super::tree::{CSGNode, CSGTree};

impl CSGTree {
    
    pub fn set_all_aabbs(&mut self, padding: f32) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        let mut propergate_ids = vec![];
        for (i, node) in self.nodes.iter_mut().enumerate() {
            match &node.data {
                CSGNodeData::Box(mat, ..) => {
                    node.aabb = AABB::from_box(mat, padding);
                    propergate_ids.push(i);
                },
                CSGNodeData::Sphere(mat, ..) => {
                    node.aabb = AABB::from_sphere(mat, padding);
                    propergate_ids.push(i);
                },
                CSGNodeData::VoxelGrid(mat, grid) => {
                    node.aabb = AABB::from_box(&mat.mul_mat4(&Mat4::from_scale(grid.size.as_vec3())), padding);
                    propergate_ids.push(i);
                }
                _ => {}
            }
        }

        propergate_ids = self.get_id_parents(&propergate_ids);
        while !propergate_ids.is_empty() {
            for id in propergate_ids.iter() {
                self.propergate_aabb_change(*id);
                // debug!("{:?}", self.nodes[*id]);
            }

            propergate_ids = self.get_id_parents(&propergate_ids);
        }
    }

    pub fn propergate_aabb_change(&mut self, i: usize) {
        let node = self.nodes[i].to_owned();
        match node.data {
            CSGNodeData::Union(child1, child2) => {
                self.nodes[i].aabb = self.nodes[child1].aabb.union(self.nodes[child2].aabb);
            }
            CSGNodeData::Remove(child1, child2) => {
                self.nodes[i].aabb = self.nodes[child1].aabb;
            }
            CSGNodeData::Intersect(child1, child2) => {
                self.nodes[i].aabb = self.nodes[child1].aabb.intersect(self.nodes[child2].aabb);
            }
            _ => {
                panic!("propergate_aabb_change can only be called for Union, Remove or Intersect")
            }
        }
    }
}

