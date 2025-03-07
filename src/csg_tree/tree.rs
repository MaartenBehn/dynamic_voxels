use crate::aabb::AABB;
use crate::csg_tree::controller::MAX_CSG_TREE_DATA_SIZE;
use crate::voxel::grid::VoxelGrid;
use octa_force::glam::{ivec3, uvec3, vec3, EulerRot, IVec3, Mat4, Quat, UVec3, Vec3};
use octa_force::log::{error, info};
use octa_force::puffin_egui::puffin;
use std::f32::consts::PI;
use std::{slice, usize};

pub const AABB_PADDING: f32 = 0.0;
pub const VOXEL_SIZE: f32 = 10.0;
pub type Material = usize;
pub const MATERIAL_NONE: usize = 0;
pub const MATERIAL_BASE: usize = 1;
pub const CSG_PARENT_NONE: usize = usize::MAX;

#[derive(Clone, Debug)]
pub struct CSGNode {
    pub data: CSGNodeData,
    pub aabb: AABB,
    pub parent: usize,
}

#[derive(Clone, Debug)]
pub enum CSGNodeData {
    Union(usize, usize),
    Remove(usize, usize),
    Intersect(usize, usize),
    Mat(Mat4, usize),
    Box(Mat4, Material),
    Sphere(Mat4, Material),
    All(Material),
    VoxelGrid(VoxelGrid, IVec3),
}

#[derive(Clone, Debug)]
pub struct CSGTree {
    pub nodes: Vec<CSGNode>,
}

impl CSGTree { 
    pub fn from_node(node: CSGNode) -> Self {
        CSGTree {
            nodes: vec![node],
        }
    }

    pub fn new_example_tree(time: f32) -> Self {
        puffin::profile_function!();

        let frac = simple_easing::roundtrip((time * 0.1) % 1.0);
        let frac_2 = simple_easing::roundtrip((time * 0.2) % 1.0);
        let frac_3 = simple_easing::roundtrip((time * 0.01) % 1.0);

        let nodes = vec![
            CSGNode::new(CSGNodeData::Union(1, 4)),
            CSGNode::new(CSGNodeData::Remove(2, 3)),
            CSGNode::new(CSGNodeData::Box(
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
                MATERIAL_BASE,
            )),
            CSGNode::new(CSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    (vec3(2.0, 1.0, 3.0) + simple_easing::cubic_in_out(frac_2) * 2.0) * VOXEL_SIZE,
                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                    vec3(2.0 + frac_3, 1.0 + frac_2 * 10.0, 0.0) * VOXEL_SIZE,
                ),
                MATERIAL_BASE,
            )),
            CSGNode::new(CSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    (vec3(3.0, 3.0, 1.0) + simple_easing::back_in_out(frac) * 10.0) * VOXEL_SIZE,
                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                    vec3(10.0 + frac * 30.0, 10.0 + frac_3 * 100.0, 0.0) * VOXEL_SIZE,
                ),
                MATERIAL_BASE,
            )),
        ];

        /*
        self.nodes = vec![
            CSGNode::Union(1, 2, Material::None, AABB::default()),

            CSGNode::Box(Mat4::from_scale_rotation_translation(
                vec3(2.0, 5.0, 7.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(0.0, 0.0, 0.0)
            ), AABB::default()),

            CSGNode::Sphere(Mat4::from_scale_rotation_translation(
                vec3(2.0, 1.0, 3.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(0.0, 0.0, 0.0)
            ), AABB::default()),
        ];

         */

        let mut tree = CSGTree {
            nodes,
        };

        tree
    }

    pub fn new_example_tree_2(time: f32) -> Self {
        puffin::profile_function!();

        let frac = simple_easing::roundtrip((time * 0.1) % 1.0);

        let mut grid = VoxelGrid::new(uvec3(256, 256,256));
        grid.set_example_sphere();

        let nodes = vec![
            CSGNode::new(CSGNodeData::Mat(Mat4::from_scale_rotation_translation(
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
            CSGNode::new(CSGNodeData::Union(2, 3)),
            CSGNode::new(CSGNodeData::Sphere(
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
                MATERIAL_BASE,
            )),
            CSGNode::new(CSGNodeData::VoxelGrid(grid, ivec3(-10, -30, 20))),
        ];

        let mut tree = CSGTree {
            nodes,
        };

        tree
    }

    pub fn new_example_tree_large(time: f32) -> Self {
        puffin::profile_function!();

        let mut grid = VoxelGrid::new(uvec3(4, 4,4));
        grid.set_example_sphere();

        let nodes = vec![
            CSGNode::new(CSGNodeData::Union(1, 2)),
            CSGNode::new(CSGNodeData::Box(
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
                MATERIAL_BASE,
            )),
            CSGNode::new(CSGNodeData::VoxelGrid(grid, ivec3(-10, -30, 200))),
        ];

        let mut tree = CSGTree {
            nodes,
        };
        tree.set_parents(0, CSG_PARENT_NONE);

        tree
    }

    pub fn set_parents(&mut self, i: usize, parent: usize) {
        self.nodes[i].parent = parent;
        match self.nodes[i].data {
            CSGNodeData::Union(c1, c2)
            | CSGNodeData::Remove(c1, c2)
            | CSGNodeData::Intersect(c1, c2) => {
                self.set_parents(c1, i);
                self.set_parents(c2, i);
            },
            CSGNodeData::Mat(_, c1) => {
                self.set_parents(c1, i);
            },
            _ => {}
        } 
    }
}

impl Default for CSGTree {
    fn default() -> Self {
        CSGTree {
            nodes: vec![],
        }
    }
}



impl CSGNode {
    pub fn new(data: CSGNodeData) -> CSGNode {
        CSGNode {
            data,
            aabb: Default::default(),
            parent: CSG_PARENT_NONE,
        }
    }

    pub fn new_with_aabb(data: CSGNodeData, aabb: AABB) -> CSGNode {
        CSGNode { data, aabb, parent: CSG_PARENT_NONE }
    }
}
