use octa_force::{glam::Mat4, puffin_egui::puffin};
use slotmap::Key;

use crate::{util::{aabb3d::AABB, iaabb3d::AABBI}, volume::{VolumeBoundsI, VolumeBounds}};

use super::tree::{CSGNodeData, CSGTree, CSGTreeKey};

impl<T: Clone> VolumeBounds for CSGTree<T> {
    fn calculate_bounds(&mut self) {
        self.set_all_aabbs()
    }

    fn get_bounds(&self) -> AABB {
        let node = &self.nodes[self.root_node];

        node.aabb
    }
}

impl<T: Clone> VolumeBoundsI for CSGTree<T> {
    fn calculate_bounds(&mut self) {
        self.set_all_aabbs()
    }

    fn get_bounds_i(&self) -> AABBI {
        let node = &self.nodes[self.root_node];
        node.aabbi
    }
}

impl<T: Clone> CSGTree<T> {
    
    pub fn set_all_aabbs(&mut self) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        let mut propergate_ids = vec![];
        self.set_primitive_aabbs(self.root_node, &Mat4::IDENTITY, &mut propergate_ids);
        
        propergate_ids = self.get_id_parents(&propergate_ids);
        while !propergate_ids.is_empty() {
            let mut new_ids = vec![];
            for id in propergate_ids.iter() {
                let parent = self.propergate_aabb_change(*id);
                if !parent.is_null() {
                    new_ids.push(parent);
                }
            }

            propergate_ids = new_ids;
        }
    }

    pub fn set_primitive_aabbs(&mut self, i: CSGTreeKey, base_mat: &Mat4, changed_nodes: &mut Vec<CSGTreeKey>) {
        let node = &self.nodes[i];
        match &node.data {
            CSGNodeData::Union(c1, c2)
                    | CSGNodeData::Remove(c1, c2) 
                    | CSGNodeData::Intersect(c1, c2) => {
                        let (c1, c2) = (*c1, *c2); // To please borrow checker
                        self.set_primitive_aabbs(c1, base_mat, changed_nodes);
                        self.set_primitive_aabbs(c2, base_mat, changed_nodes);
                    },
            CSGNodeData::Box(mat, ..) => {
                        let mat = mat.inverse().mul_mat4(base_mat);
                        self.nodes[i].set_aabb(AABB::from_box(&mat));
                        changed_nodes.push(i);
                    },
            CSGNodeData::Sphere(mat, ..) => {
                        let mat = mat.inverse().mul_mat4(base_mat);
                        self.nodes[i].set_aabb(AABB::from_sphere(&mat));
                        changed_nodes.push(i);
                    },
            CSGNodeData::All(_) => {
                        self.nodes[i].set_aabb(AABB::infinte());
                        changed_nodes.push(i);
                    },
            CSGNodeData::OffsetVoxelGrid(grid) => {
                let aabb = grid.get_bounds(); 
                self.nodes[i].set_aabb(aabb);
                changed_nodes.push(i);
            },
            CSGNodeData::SharedVoxelGrid(grid) => {
                let aabb = grid.get_bounds();
                self.nodes[i].set_aabb(aabb);
                changed_nodes.push(i);
            },
        }
    }

    pub fn propergate_aabb_change(&mut self, i: CSGTreeKey) -> CSGTreeKey {
        let node = self.nodes[i].to_owned();
        match node.data {
            CSGNodeData::Union(c1, c2) => {
                let aabb = self.nodes[c1].aabb.union(self.nodes[c2].aabb);
                self.nodes[i].set_aabb(aabb);
            }
            CSGNodeData::Remove(c1, c2) => {
                let aabb = self.nodes[c1].aabb;
                self.nodes[i].set_aabb(aabb);
            }
            CSGNodeData::Intersect(c1, c2) => {
                let aabb = self.nodes[c1].aabb.intersect(self.nodes[c2].aabb);
                self.nodes[i].set_aabb(aabb);
            }
            _ => {
                panic!("propergate_aabb_change can only be called for Union, Remove or Intersect")
            }
        }

        node.parent
    }
}
