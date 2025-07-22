use octa_force::glam::{vec2, Affine2, Vec2};

use crate::csg::Base;

use super::tree::{CSGNode2D, CSGNodeData2D, CSGTree2D};

impl<T: Base + Clone> CSGTree2D<T> {
    pub fn new_circle(center: Vec2, radius: f32) -> Self {
        CSGTree2D::from_node(CSGNode2D::new_circle(center, radius))
    } 
}

impl <T: Base + Clone> CSGNode2D<T> {
    pub fn new_circle(center: Vec2, radius: f32) -> Self {
        CSGNode2D::new(CSGNodeData2D::Circle(
            Affine2::from_scale_angle_translation(Vec2::splat(radius), 0.0,center).inverse(),
            T::base(),
        ))
    } 
} 
