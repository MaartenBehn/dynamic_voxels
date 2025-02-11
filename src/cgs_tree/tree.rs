use crate::aabb::AABB;
use crate::cgs_tree::controller::MAX_CGS_TREE_DATA_SIZE;
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
    VoxelVolume(Material),
}

#[derive(Clone, Debug)]
pub struct CSGTree {
    pub nodes: Vec<CSGNode>,
}

impl CSGTree {
    pub fn new() -> Self {
        CSGTree {
            nodes: vec![],
        }
    }

    pub fn set_example_tree(&mut self, time: f32) {
        puffin::profile_function!();

        let frac = simple_easing::roundtrip((time * 0.1) % 1.0);
        let frac_2 = simple_easing::roundtrip((time * 0.2) % 1.0);
        let frac_3 = simple_easing::roundtrip((time * 0.01) % 1.0);

        self.nodes = vec![
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
    }

    pub fn set_example_tree_with_aabb_field(&mut self, time: f32) {
        puffin::profile_function!();

        let frac = simple_easing::roundtrip((time * 0.1) % 1.0);

        self.nodes = vec![
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
    }

    pub fn set_all_aabbs(&mut self, padding: f32) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        let mut propergate_ids = vec![];
        for (i, node) in self.nodes.iter_mut().enumerate() {
            match node.data {
                CSGNodeData::Box(..) | CSGNodeData::Sphere(..) => {
                    Self::set_leaf_aabb(node, padding);
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

    pub fn get_id_parents(&self, ids: &[usize]) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                match node.data {
                    CSGNodeData::Union(child1, child2)
                    | CSGNodeData::Remove(child1, child2)
                    | CSGNodeData::Intersect(child1, child2) => {
                        if ids.contains(&child1) || ids.contains(&child2) {
                            return Some(i);
                        }
                    }
                    _ => {}
                }

                None
            })
            .collect()
    }

    pub fn set_leaf_aabb(node: &mut CSGNode, padding: f32) {
        match node.data {
            CSGNodeData::Box(mat, ..) => node.aabb = AABB::from_box(&mat, padding),
            CSGNodeData::Sphere(mat, ..) => node.aabb = AABB::from_sphere(&mat, padding),
            _ => {
                panic!("set_leaf_aabb can only be called for Box or Sphere")
            }
        }
    } 
}

impl Default for CSGTree {
    fn default() -> Self {
        Self::new()
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
