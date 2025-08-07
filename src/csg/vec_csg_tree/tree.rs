use octa_force::glam::{ivec3, uvec3, vec3, EulerRot, IVec3, Mat4, Quat, UVec3, Vec3};
use octa_force::log::{error, info};
use octa_force::puffin_egui::puffin;
use std::f32::consts::PI;
use std::{slice, usize};

use crate::csg::Base;
use crate::util::aabb3d::AABB;
use crate::voxel::grid::VoxelGrid;
use crate::voxel::renderer::palette::MATERIAL_ID_BASE;

pub const VOXEL_SIZE: f32 = 10.0;

pub const CSG_PARENT_NONE: usize = usize::MAX;

#[derive(Clone, Debug)]
pub struct VecCSGNode<T> {
    pub data: VecCSGNodeData<T>,
    pub aabb: AABB,
    pub parent: usize,
}

#[derive(Clone, Debug)]
pub enum VecCSGNodeData<T> {
    Union(usize, usize),
    Remove(usize, usize),
    Intersect(usize, usize),
    Mat(Mat4, usize),
    Box(Mat4, T),
    Sphere(Mat4, T),
    All(T),
    VoxelGrid(VoxelGrid, IVec3),
}

#[derive(Clone, Debug)]
pub struct VecCSGTree<T> {
    pub nodes: Vec<VecCSGNode<T>>,
}

impl<T> VecCSGTree<T> { 
    pub fn from_node(node: VecCSGNode<T>) -> Self {
        VecCSGTree {
            nodes: vec![node],
        }
    }
 
    pub fn set_parents(&mut self, i: usize, parent: usize) {
        self.nodes[i].parent = parent;
        match self.nodes[i].data {
            VecCSGNodeData::Union(c1, c2)
            | VecCSGNodeData::Remove(c1, c2)
            | VecCSGNodeData::Intersect(c1, c2) => {
                self.set_parents(c1, i);
                self.set_parents(c2, i);
            },
            VecCSGNodeData::Mat(_, c1) => {
                self.set_parents(c1, i);
            },
            _ => {}
        } 
    }
}

impl<T: Base + Clone> VecCSGTree<T> {
    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        let nodes = vec![
            VecCSGNode::new(VecCSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    Vec3::ONE * radius,
                    Quat::IDENTITY,
                    center,
                ),
                T::base(),
            )),
        ];
 
        let mut tree = VecCSGTree {
            nodes,
        };

        tree.set_parents(0, CSG_PARENT_NONE);
        tree.set_all_aabbs();

        tree
    } 

    pub fn new_example_tree(time: f32) -> Self {
        puffin::profile_function!();

        let frac = simple_easing::roundtrip((time * 0.1) % 1.0);
        let frac_2 = simple_easing::roundtrip((time * 0.2) % 1.0);
        let frac_3 = simple_easing::roundtrip((time * 0.01) % 1.0);

        let nodes = vec![
            VecCSGNode::new(VecCSGNodeData::Union(1, 4)),
            VecCSGNode::new(VecCSGNodeData::Remove(2, 3)),
            VecCSGNode::new(VecCSGNodeData::Box(
                Mat4::from_scale_rotation_translation(
                    (vec3(2.0, 5.0, 7.0) + simple_easing::expo_in_out(frac)) * VOXEL_SIZE,
                    Quat::from_euler(
                        EulerRot::XYZ,
                        (time * 0.1) % (2.0 * PI),
                        (time * 0.11) % (2.0 * PI),
                        0.0,
                    ),
                    vec3(3.0, 3.0, 0.0) * VOXEL_SIZE,
                ),
                T::base(),
            )),
            VecCSGNode::new(VecCSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    (vec3(2.0, 1.0, 3.0) + simple_easing::cubic_in_out(frac_2) * 2.0) * VOXEL_SIZE,
                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                    vec3(2.0 + frac_3, 1.0 + frac_2 * 10.0, 0.0) * VOXEL_SIZE,
                ),
                T::base(),
            )),
            VecCSGNode::new(VecCSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    (vec3(3.0, 3.0, 1.0) + simple_easing::back_in_out(frac) * 10.0) * VOXEL_SIZE,
                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                    vec3(10.0 + frac * 30.0, 10.0 + frac_3 * 100.0, 0.0) * VOXEL_SIZE,
                ),
                T::base(),
            )),
        ];
 
        let mut tree = VecCSGTree {
            nodes,
        };

        tree.set_parents(0, CSG_PARENT_NONE);
        tree.set_all_aabbs();

        tree
    }

    pub fn new_disk(center: Vec3, radius: f32, height: f32) -> Self {
        let nodes = vec![
            VecCSGNode::new(VecCSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    vec3(radius, height, radius),
                    Quat::IDENTITY,
                    center,
                ),
                T::base(),
            )),
        ];
 
        let mut tree = VecCSGTree {
            nodes,
        };

        tree.set_parents(0, CSG_PARENT_NONE);
        tree.set_all_aabbs();

        tree
    }

    pub fn new_example_tree_2(time: f32) -> Self {
        puffin::profile_function!();

        let frac = simple_easing::roundtrip((time * 0.1) % 1.0);

        let mut grid = VoxelGrid::empty(uvec3(256, 256,256));
        grid.set_example_sphere();

        let nodes = vec![
            VecCSGNode::new(VecCSGNodeData::Mat(Mat4::from_scale_rotation_translation(
                    vec3(1.0, 1.0, 1.0),
                    Quat::from_euler(
                        EulerRot::XYZ,
                        (time * 0.3) % (2.0 * PI),
                        (time * 0.5) % (2.0 * PI),
                        0.0,
                    ),
                    vec3(0.0, 0.0, 0.0) * VOXEL_SIZE,
                ),
                1)),
            VecCSGNode::new(VecCSGNodeData::Union(2, 3)),
            VecCSGNode::new(VecCSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    (vec3(2.0, 5.0, 7.0) + simple_easing::expo_in_out(frac)) * VOXEL_SIZE,
                    Quat::from_euler(
                        EulerRot::XYZ,
                        (time * 0.3) % (2.0 * PI),
                        (time * 0.5) % (2.0 * PI),
                        0.0,
                    ),
                    vec3(20.0, 10.0, 10.0) * VOXEL_SIZE,
                ),
                T::base(),
            )),
            VecCSGNode::new(VecCSGNodeData::VoxelGrid(grid, ivec3(-10, -30, 20))),
        ];

        let mut tree = VecCSGTree {
            nodes,
        };

        tree.set_parents(0, CSG_PARENT_NONE);
        tree.set_all_aabbs();

        tree
    }

    pub fn new_example_tree_large(time: f32) -> Self {
        puffin::profile_function!();

        let mut grid = VoxelGrid::empty(uvec3(4, 4,4));
        grid.set_example_sphere();

        let nodes = vec![
            VecCSGNode::new(VecCSGNodeData::Union(1, 2)),
            VecCSGNode::new(VecCSGNodeData::Box(
                Mat4::from_scale_rotation_translation(
                    vec3(0.0, 0.0, 0.0) * VOXEL_SIZE,
                    Quat::from_euler(
                        EulerRot::XYZ,
                        0.0,
                        0.0,
                        0.0,
                    ),
                    vec3(20.0, 10.0, 10.0) * VOXEL_SIZE,
                ),
                T::base(),
            )),
            VecCSGNode::new(VecCSGNodeData::VoxelGrid(grid, ivec3(-10, -30, 200))),
        ];

        let mut tree = VecCSGTree {
            nodes,
        };
        tree.set_parents(0, CSG_PARENT_NONE);
        tree.set_all_aabbs();

        tree
    }
}

impl<T> Default for VecCSGTree<T> {
    fn default() -> Self {
        VecCSGTree {
            nodes: vec![],
        }
    }
}



impl<T> VecCSGNode<T> {
    pub fn new(data: VecCSGNodeData<T>) -> VecCSGNode<T> {
        VecCSGNode {
            data,
            aabb: Default::default(),
            parent: CSG_PARENT_NONE,
        }
    }

    pub fn new_with_aabb(data: VecCSGNodeData<T>, aabb: AABB) -> VecCSGNode<T> {
        VecCSGNode { data, aabb, parent: CSG_PARENT_NONE }
    }
}
