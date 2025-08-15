use octa_force::glam::{ivec3, uvec3, vec2, vec3, vec4, Affine2, EulerRot, IVec3, Mat4, Quat, UVec3, Vec2, Vec3, Vec3A, Vec4, Vec4Swizzles};
use octa_force::log::{error, info};
use octa_force::puffin_egui::puffin;
use slotmap::{new_key_type, Key, SlotMap};
use std::f32::consts::PI;
use std::{slice, usize};

use crate::csg::Base;
use crate::util::aabb2d::AABB2;
use crate::util::aabb3d::AABB3;
use crate::volume::VolumeQureyAABBResult;

new_key_type! { pub struct CSGTreeKey2D; }

#[derive(Clone, Copy, Debug)]
pub enum CSGNodeData2D<T> {
    Union(CSGTreeKey2D, CSGTreeKey2D),
    Remove(CSGTreeKey2D, CSGTreeKey2D),
    Intersect(CSGTreeKey2D, CSGTreeKey2D),
    Box(Affine2, T),
    Circle(Affine2, T),
    All(T),
}

#[derive(Clone, Debug)]
pub struct CSGNode2D<T> {
    pub data: CSGNodeData2D<T>,
    pub aabb: AABB2,
    pub parent: CSGTreeKey2D,
}

#[derive(Clone, Debug, Default)]
pub struct CSGTree2D<T> {
    pub nodes: SlotMap<CSGTreeKey2D, CSGNode2D<T>>,
    pub root_node: CSGTreeKey2D,
}

impl<T> CSGNode2D<T> {
    pub fn new(data: CSGNodeData2D<T>) -> Self {
        CSGNode2D {
            data,
            aabb: Default::default(),
            parent: CSGTreeKey2D::null(),
        }
    }
}

impl<T: Clone> CSGTree2D<T> {
    pub fn is_empty(&self) -> bool {
        self.root_node == CSGTreeKey2D::null()
    }

    pub fn from_node(node: CSGNode2D<T>) -> Self {
        let mut tree = Self {
            nodes: SlotMap::with_capacity_and_key(1),
            root_node: CSGTreeKey2D::null(),
        };
        let index = tree.nodes.insert(node);
        tree.root_node = index;

        tree.set_all_aabbs();

        tree
    }
}

