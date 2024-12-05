use std::{slice};
use std::f32::consts::PI;
use fastrand::Rng;
use octa_force::egui::Key::C;
use octa_force::glam::{ivec3, uvec3, vec3, EulerRot, Mat4, Quat};
use octa_force::log::{error, info};
use octa_force::puffin_egui::puffin;
use crate::aabb::AABB;

const CGS_TYPE_BOX: u32 = 0;
const CGS_TYPE_SPHERE: u32 =  1;
#[allow(dead_code)]
const CGS_TYPE_CAPSULE: u32 =  2;

const CGS_CHILD_TYPE_GEO: u32 =  0;
const CGS_CHILD_TYPE_UNION: u32 =  1;
const CGS_CHILD_TYPE_REMOVE: u32 =  2;
const CGS_CHILD_TYPE_INTERSECT: u32 =  3;

pub const MAX_CGS_TREE_DATA_SIZE: usize = 100;

const AABB_PADDING: f32 = 2.0;
pub const VOXEL_SIZE: f32 = 10.0;

#[derive(Copy, Clone, Debug)]
pub enum Material {
    None,
    BASE,
    RED, 
    YELLOW
}

#[derive(Clone, Debug)]
pub struct CSGNode {
    data: CSGNodeData,
    aabb: AABB
}

#[derive(Clone, Debug)]
pub enum CSGNodeData {
    Union(usize, usize, Material),
    Remove(usize, usize, Material),
    Intersect(usize, usize, Material),
    Box(Mat4),
    Sphere(Mat4),
    VoxelVolume(Vec<u8>)
}

#[derive(Clone)]
pub struct CSGTree {
    pub data: Vec<u32>,
    pub nodes: Vec<CSGNode>,
}

impl CSGTree {
    pub fn new() -> Self {
        CSGTree {
            data: vec![],
            nodes: vec![],
        }
    }
    
    pub fn set_example_tree(&mut self, time: f32) {
        puffin::profile_function!();
        
        let frac = simple_easing::roundtrip((time * 0.1) % 1.0);
        let frac_2 = simple_easing::roundtrip((time * 0.2) % 1.0);
        let frac_3 = simple_easing::roundtrip((time * 0.01) % 1.0);
        
        self.nodes = vec![
            CSGNode::new(CSGNodeData::Union(1, 4, Material::None)),
            CSGNode::new(CSGNodeData::Remove(2, 3, Material::None)),
            
            CSGNode::new(CSGNodeData::Box(Mat4::from_scale_rotation_translation(
                (vec3(2.0, 5.0 , 7.0) + simple_easing::expo_in_out(frac)) * VOXEL_SIZE,
                Quat::from_euler(EulerRot::XYZ, (time * 0.1) % (2.0 * PI),(time * 0.11) % (2.0 * PI),0.0),
                vec3(3.0, 3.0, 0.0)  * VOXEL_SIZE
            ))),

            CSGNode::new(CSGNodeData::Sphere(Mat4::from_scale_rotation_translation(
                (vec3(2.0, 1.0, 3.0) + simple_easing::cubic_in_out(frac_2) * 2.0) * VOXEL_SIZE,
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(2.0 + frac_3, 1.0 + frac_2, 0.0)  * VOXEL_SIZE
            ))),
        
            CSGNode::new(CSGNodeData::Sphere(Mat4::from_scale_rotation_translation(
                (vec3(3.0, 3.0, 1.0) + simple_easing::back_in_out(frac) * 10.0) * VOXEL_SIZE,
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(10.0 + frac * 30.0, 10.0 + frac_3 * 100.0, 0.0) * VOXEL_SIZE
            ))),
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
    
    pub fn set_example_voxel_volume(&mut self) {
        
        let voxels = vec![];
        let center = vec3(5.0, 5.0, 5.0);
        let radius = 3.0;
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    let index = x * 100 + y * 10 + z;
                    let pos = vec3(x as f32, y as f32, z as f32); 
                    let dist = (center - pos).length();
                    
                    if dist < radius {
                        voxels[index] = 1;
                    } else {
                        voxels[index] = 0;
                    }
                    
                    
                }
            }
        }
        
        
        self.nodes = vec![
            CSGNode::new(CSGNodeData::Union(1, 4, Material::None)),
            CSGNode::new(CSGNodeData::VoxelVolume()),
            
        ];
    }
    
    pub fn set_all_aabbs(&mut self) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        
        let mut propergate_ids = vec![];
        for (i, node) in self.nodes.iter_mut().enumerate() {
            match node.data {
                CSGNodeData::Box(_) | CSGNodeData::Sphere(_) => {
                    Self::set_leaf_aabb(node);
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
            CSGNodeData::Union(child1, child2, _) 
            | CSGNodeData::Remove(child1, child2, _)
            | CSGNodeData::Intersect(child1, child2, _) => {
                self.nodes[i].aabb = self.nodes[child1].aabb.merge(self.nodes[child2].aabb);
            }
            _ => {panic!("propergate_aabb_change can only be called for Union, Remove or Intersect")}
        }
    }

    pub fn get_id_parents(&self, ids: &[usize]) -> Vec<usize> {
        self.nodes.iter()
            .enumerate()
            .filter_map(|(i, node)| {
                match node.data {
                    CSGNodeData::Union(child1, child2, _)
                    | CSGNodeData::Remove(child1, child2, _)
                    | CSGNodeData::Intersect(child1, child2, _) => {
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

    pub fn set_leaf_aabb(node: &mut CSGNode) {
        match node.data {
            CSGNodeData::Box(mat) => { node.aabb = AABB::from_box(&mat, AABB_PADDING) }
            CSGNodeData::Sphere(mat) => { node.aabb = AABB::from_sphere(&mat, AABB_PADDING) }
            _ => {panic!("set_leaf_aabb can only be called for Box or Sphere")}
        }
    }
    
    pub fn make_data(&mut self) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        
        let (data, _) = self.add_data(0, vec![]);

        if data.len() > MAX_CGS_TREE_DATA_SIZE {
            error!("CGS Tree Data to large: {} of {}", data.len(), MAX_CGS_TREE_DATA_SIZE)
        }
        
        self.data = data;
       
    }
    
    fn add_data(&self, index: usize, mut data: Vec<u32>) -> (Vec<u32>, u32) {
        let node = &self.nodes[index];
        
        let node_data = match node.data {
            CSGNodeData::Union(child1, child2, mat) 
                | CSGNodeData::Remove(child1, child2, mat) 
                | CSGNodeData::Intersect(child1, child2, mat)
            => {
                
                let index = data.len();
                data.push(0);
                data.push(0);
                data.extend_from_slice(any_as_u32_slice(&node.aabb));

                (data, data[index]) = self.add_data(child1, data);
                (data, data[index + 1]) = self.add_data(child2, data);
                
                let t = match node.data {
                    CSGNodeData::Union(_, _, _) => { CGS_CHILD_TYPE_UNION }
                    CSGNodeData::Remove(_, _, _) => { CGS_CHILD_TYPE_REMOVE }
                    CSGNodeData::Intersect(_, _, _) => { CGS_CHILD_TYPE_INTERSECT }
                    _ => unreachable!()
                };
                Self::node_data(index, t, mat)
            }
            CSGNodeData::Box(transform) | CSGNodeData::Sphere(transform) => {
                let index = data.len();

                let t = match node.data {
                    CSGNodeData::Box(_) => {CGS_TYPE_BOX}
                    CSGNodeData::Sphere(_) => {CGS_TYPE_SPHERE}
                    _ => unreachable!()
                };
                
                data.extend_from_slice(any_as_u32_slice(&transform.inverse()));
                data[index + 15] = t;
                data.extend_from_slice(any_as_u32_slice(&node.aabb));
                
                Self::node_data(index, CGS_CHILD_TYPE_GEO, Material::None)
            },
            CSGNodeData::VoxelVolume(_) => todo!()
        };
        
        (data, node_data)
    }
    
    fn node_data(pointer: usize, t: u32, mat: Material) -> u32 {
        ((pointer as u32) << 16) + (t << 6) + (mat as u32)
    }
}

fn any_as_u32_slice<T: Sized>(p: &T) -> &[u32] {
    unsafe {
        slice::from_raw_parts(
            (p as *const T) as *const u32,
            size_of::<T>() / 4,
        )
    }
}

impl CSGNode {
    
    pub fn new(data: CSGNodeData) -> CSGNode {
        CSGNode {
            data,
            aabb: Default::default(),
        }    
    }
}