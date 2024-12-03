use std::{slice};
use log::{debug, error};
use octa_force::glam::{vec3, EulerRot, Mat4, Quat};
use crate::aabb::AABB;

const CGS_TYPE_BOX: u32 = 0;
const CGS_TYPE_SPHERE: u32 =  1;
#[allow(dead_code)]
const CGS_TYPE_CAPSULE: u32 =  2;

const CGS_CHILD_TYPE_GEO: u32 =  0;
const CGS_CHILD_TYPE_UNION: u32 =  1;
const CGS_CHILD_TYPE_REMOVE: u32 =  2;
const CGS_CHILD_TYPE_INTERSECT: u32 =  3;

pub const MAX_CGS_TREE_DATA_SIZE: usize = 1024;

const AABB_PADDING: f32 = 0.0;

#[derive(Copy, Clone, Debug)]
pub enum Material {
    None,
    BASE,
    RED, 
    YELLOW
}

#[derive(Copy, Clone, Debug)]
pub enum CSGNode {
    Union(usize, usize, Material, AABB),
    Remove(usize, usize, Material, AABB),
    Intersect(usize, usize, Material, AABB),
    Box(Mat4, AABB),
    Sphere(Mat4, AABB),
}

#[derive(Clone)]
pub struct CGSTree {
    pub data: Vec<u32>,
    pub nodes: Vec<CSGNode>,
}

impl CGSTree {
    pub fn new() -> Self {
        CGSTree {
            data: vec![],
            nodes: vec![],
        }
    }
    
    pub fn set_example_tree(&mut self) {
        self.nodes = vec![
            CSGNode::Union(1, 4, Material::None, AABB::default()),
            CSGNode::Remove(2, 3, Material::None, AABB::default()),
            
            CSGNode::Box(Mat4::from_scale_rotation_translation(
                vec3(2.0, 5.0, 7.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(5.0, 0.0, 0.0)
            ), AABB::default()),

            CSGNode::Sphere(Mat4::from_scale_rotation_translation(
                vec3(2.0, 1.0, 3.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(5.0, 1.0, 0.0)
            ), AABB::default()),
        
            CSGNode::Sphere(Mat4::from_scale_rotation_translation(
                vec3(3.0, 3.0, 1.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(0.0, 0.0, 0.0)
            ), AABB::default()),
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
        
        
        
        self.set_all_aabbs();
    }
    
    pub fn set_all_aabbs(&mut self) {
        
        let mut propergate_ids = vec![];
        for (i, node) in self.nodes.iter_mut().enumerate() {
            match node {
                CSGNode::Box(_, _) | CSGNode::Sphere(_, _) => {
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
        match node {
            CSGNode::Union(child1, child2, _, aabb) 
            | CSGNode::Remove(child1, child2, _, aabb)
            | CSGNode::Intersect(child1, child2, _, aabb) => {
                *self.nodes[i].aabb_mut() = self.nodes[child1].aabb().merge(self.nodes[child2].aabb());
            }
            _ => {panic!("propergate_aabb_change can only be called for Union, Remove or Intersect")}
        }
    }

    pub fn get_id_parents(&self, ids: &[usize]) -> Vec<usize> {
        self.nodes.iter()
            .enumerate()
            .filter_map(|(i, node)| {
                match node {
                    CSGNode::Union(child1, child2, _, aabb)
                    | CSGNode::Remove(child1, child2, _, aabb)
                    | CSGNode::Intersect(child1, child2, _, aabb) => {
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
        match node {
            CSGNode::Box(mat, aabb) => { *aabb = AABB::from_box(mat, AABB_PADDING) }
            CSGNode::Sphere(mat, aabb) => { *aabb = AABB::from_sphere(mat, AABB_PADDING) }
            _ => {panic!("set_leaf_aabb can only be called for Box or Sphere")}
        }
    }
    
    pub fn make_data(&mut self) {
        let (data, _) = self.add_data(0, vec![]);
        self.data = data;

        debug!("{:?}", self.data);
    }
    
    fn add_data(&self, index: usize, mut data: Vec<u32>) -> (Vec<u32>, u32) {
        let node = &self.nodes[index];
        
        let node_data = match node {
            CSGNode::Union(child1, child2, mat, aabb) 
                | CSGNode::Remove(child1, child2, mat, aabb) 
                | CSGNode::Intersect(child1, child2, mat, aabb)
            => {
                
                let index = data.len();
                data.push(0);
                data.push(0);
                data.extend_from_slice(any_as_u32_slice(aabb));

                (data, data[index]) = self.add_data(*child1, data);
                (data, data[index + 1]) = self.add_data(*child2, data);
                
                let t = match node {
                    CSGNode::Union(_, _, _, _) => { CGS_CHILD_TYPE_UNION }
                    CSGNode::Remove(_, _, _, _) => { CGS_CHILD_TYPE_REMOVE }
                    CSGNode::Intersect(_, _, _, _) => { CGS_CHILD_TYPE_INTERSECT }
                    _ => unreachable!()
                };
                Self::node_data(index, t, *mat)
            }
            CSGNode::Box(transform, aabb) | CSGNode::Sphere(transform, aabb) => {
                let index = data.len();

                let t = match node {
                    CSGNode::Box(_, _) => {CGS_TYPE_BOX}
                    CSGNode::Sphere(_, _) => {CGS_TYPE_SPHERE}
                    _ => unreachable!()
                };
                
                data.extend_from_slice(any_as_u32_slice(&transform.inverse()));
                data[index + 15] = t;
                data.extend_from_slice(any_as_u32_slice(aabb));
                
                Self::node_data(index, CGS_CHILD_TYPE_GEO, Material::None)
            }
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
    pub fn aabb(&self) -> AABB {
        match self {
            CSGNode::Union(_, _, _, aabb)
            | CSGNode::Remove(_, _, _, aabb) 
            | CSGNode::Intersect(_, _, _, aabb) 
            | CSGNode::Box(_, aabb) 
            | CSGNode::Sphere(_, aabb) => {aabb.to_owned()}
        }
    }

    pub fn aabb_mut(&mut self) -> &mut AABB {
        match self {
            CSGNode::Union(_, _, _, aabb)
            | CSGNode::Remove(_, _, _, aabb)
            | CSGNode::Intersect(_, _, _, aabb)
            | CSGNode::Box(_, aabb)
            | CSGNode::Sphere(_, aabb) => {aabb}
        }
    }
}