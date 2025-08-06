use octa_force::glam::{vec3, Mat4, Quat, Vec3};
use slotmap::Key;

use crate::{csg::{vec_csg_tree::tree::VecCSGNodeData, Base}};

use super::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};


impl<T: Base + Clone> SlotMapCSGTree<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        SlotMapCSGTree::from_node(SlotMapCSGNode::new_sphere(center, radius))
    }

    pub fn new_disk(center: Vec3, radius: f32, height: f32) -> Self {
        SlotMapCSGTree::from_node(SlotMapCSGNode::new_disk(center, radius, height))
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

    pub fn new_disk(center: Vec3, radius: f32, height: f32) -> Self {
        SlotMapCSGNode::new(SlotMapCSGNodeData::Sphere(
            Mat4::from_scale_rotation_translation(
                vec3(radius, radius, height),
                Quat::IDENTITY,
                center,
            ).inverse(),
            T::base(),
        ))
    }
} 


