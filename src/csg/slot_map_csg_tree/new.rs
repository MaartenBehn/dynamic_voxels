use octa_force::glam::{Mat4, Quat, Vec3};
use slotmap::Key;

use crate::{csg::{vec_csg_tree::tree::VecCSGNodeData, Base}};

use super::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};


impl<T: Base + Clone> SlotMapCSGTree<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        SlotMapCSGTree::from_node(SlotMapCSGNode::new_sphere(center, radius))
    } 
}

impl <T: Base + Clone> SlotMapCSGNode<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        SlotMapCSGNode::new(SlotMapCSGNodeData::Sphere(
            Mat4::from_scale_rotation_translation(
                Vec3::ONE * radius,
                Quat::IDENTITY,
                center,
            ).inverse(),
            T::base(),
        ))
    } 
} 


