use crate::aabb::AABB;
use crate::csg_tree::controller::MAX_CGS_TREE_DATA_SIZE;
use octa_force::glam::{uvec3, vec3, EulerRot, Mat4, Quat, UVec3, Vec3};
use octa_force::log::{error, info};
use octa_force::puffin_egui::puffin;
use std::f32::consts::PI;
use std::slice;

pub const AABB_PADDING: f32 = 10.0;
pub const VOXEL_SIZE: f32 = 10.0;
pub type Material = usize;
pub const MATERIAL_NONE: usize = usize::MAX;

#[derive(Clone, Debug)]
pub struct CSGNode {
    pub data: CSGNodeData,
    pub aabb: AABB,
}

#[derive(Clone, Debug)]
pub enum CSGNodeData {
    Union(usize, usize),
    Remove(usize, usize),
    Intersect(usize, usize),
    Box(Mat4, Material),
    Sphere(Mat4, Material),
    All(Material),
    VoxelVolume(Material),
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
                MATERIAL_NONE,
            )),
            CSGNode::new(CSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    (vec3(2.0, 1.0, 3.0) + simple_easing::cubic_in_out(frac_2) * 2.0) * VOXEL_SIZE,
                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                    vec3(2.0 + frac_3, 1.0 + frac_2 * 10.0, 0.0) * VOXEL_SIZE,
                ),
                MATERIAL_NONE,
            )),
            CSGNode::new(CSGNodeData::Sphere(
                Mat4::from_scale_rotation_translation(
                    (vec3(3.0, 3.0, 1.0) + simple_easing::back_in_out(frac) * 10.0) * VOXEL_SIZE,
                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
                    vec3(10.0 + frac * 30.0, 10.0 + frac_3 * 100.0, 0.0) * VOXEL_SIZE,
                ),
                MATERIAL_NONE,
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

        CSGTree {
            nodes,
        }
    }

    pub fn new_example_tree_with_aabb_field(time: f32) -> Self {
        puffin::profile_function!();

        let frac = simple_easing::roundtrip((time * 0.1) % 1.0);

        let nodes = vec![
            CSGNode::new(CSGNodeData::Union(1, 2)),
            CSGNode::new(CSGNodeData::Box(
                Mat4::from_scale_rotation_translation(
                    (vec3(2.0, 5.0, 7.0) + simple_easing::expo_in_out(frac)) * VOXEL_SIZE,
                    Quat::from_euler(
                        EulerRot::XYZ,
                        (time * 0.1) % (2.0 * PI),
                        (time * 0.11) % (2.0 * PI),
                        0.0,
                    ),
                    vec3(10.0, 10.0, 0.0) * VOXEL_SIZE,
                ),
                MATERIAL_NONE,
            )),
            CSGNode::new_with_aabb(
                CSGNodeData::VoxelVolume(0),
                AABB {
                    min: Vec3::ZERO,
                    max: Vec3::ONE * 16.0,
                },
            ),
        ];

        CSGTree {
            nodes,
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
        }
    }

    pub fn new_with_aabb(data: CSGNodeData, aabb: AABB) -> CSGNode {
        CSGNode { data, aabb }
    }
}
