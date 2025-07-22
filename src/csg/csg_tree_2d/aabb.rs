use octa_force::{glam::Mat4, puffin_egui::puffin};
use slotmap::Key;

use crate::{csg::slot_map_csg_tree::tree::SlotMapCSGTreeKey, util::{aabb2d::AABB2D, aabb3d::AABB}, volume::VolumeBounds2D};

use super::tree::{CSGNodeData2D, CSGTree2D, CSGTreeKey2D};

impl<T: Clone> VolumeBounds2D for CSGTree2D<T> {
    fn calculate_bounds(&mut self) {
        self.set_all_aabbs()
    }

    fn get_bounds(&self) -> AABB2D {
        let node = &self.nodes[self.root_node];

        node.aabb
    }
}

impl<T: Clone> CSGTree2D<T> {
    
    pub fn set_all_aabbs(&mut self) {
        let mut propergate_ids = vec![];
        self.set_primitive_aabbs(self.root_node, &mut propergate_ids);
        
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

    pub fn set_primitive_aabbs(&mut self, i: CSGTreeKey2D, changed_nodes: &mut Vec<CSGTreeKey2D>) {
        let node = &self.nodes[i];
        match &node.data {
            CSGNodeData2D::Union(c1, c2)
            | CSGNodeData2D::Remove(c1, c2) 
            | CSGNodeData2D::Intersect(c1, c2) => {
                let (c1, c2) = (*c1, *c2); // To please borrow checker
                self.set_primitive_aabbs(c1, changed_nodes);
                self.set_primitive_aabbs(c2, changed_nodes);
            },
            CSGNodeData2D::Box(mat, ..) => {
                self.nodes[i].aabb = AABB2D::from_box(&mat.inverse());
                changed_nodes.push(i);
            },
            CSGNodeData2D::Circle(mat, ..) => {
                self.nodes[i].aabb = AABB2D::from_circle(&mat.inverse());
                changed_nodes.push(i);
            },
            CSGNodeData2D::All(_) => {
                self.nodes[i].aabb = AABB2D::infinte();
                changed_nodes.push(i);
            },
        }
    }

    pub fn propergate_aabb_change(&mut self, i: CSGTreeKey2D) -> CSGTreeKey2D {
        let node = self.nodes[i].to_owned();
        match node.data {
            CSGNodeData2D::Union(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb.union(self.nodes[c2].aabb);
            }
            CSGNodeData2D::Remove(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb;
            }
            CSGNodeData2D::Intersect(c1, c2) => {
                self.nodes[i].aabb = self.nodes[c1].aabb.intersect(self.nodes[c2].aabb);
            }
            _ => {
                panic!("propergate_aabb_change can only be called for Union, Remove or Intersect")
            }
        }

        node.parent
    }
}
