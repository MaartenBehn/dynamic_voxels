use std::{slice};
use octa_force::glam::{vec3, EulerRot, Mat4, Quat};

const CGS_TYPE_BOX: u32 = 0;
const CGS_TYPE_SPHERE: u32 =  1;
#[allow(dead_code)]
const CGS_TYPE_CAPSULE: u32 =  2;

const CGS_CHILD_TYPE_GEO: u32 =  0;
const CGS_CHILD_TYPE_UNION: u32 =  1;
const CGS_CHILD_TYPE_REMOVE: u32 =  2;
const CGS_CHILD_TYPE_INTERSECT: u32 =  3;

pub const MAX_CGS_TREE_DATA_SIZE: usize = 1024;

#[derive(Copy, Clone)]
pub enum Material {
    None,
    BASE,
    RED, 
    YELLOW
}

#[derive(Copy, Clone)]
pub enum CSGNode {
    Union(usize, usize, Material),
    Remove(usize, usize, Material),
    Intersect(usize, usize, Material),
    Box(Mat4),
    Sphere(Mat4),
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
            CSGNode::Union(1, 4, Material::None),
            CSGNode::Remove(2, 3, Material::None),
            
            CSGNode::Box(Mat4::from_scale_rotation_translation(
                vec3(2.0, 5.0, 7.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(5.0, 0.0, 0.0)
            ).inverse()),

            CSGNode::Sphere(Mat4::from_scale_rotation_translation(
                vec3(2.0, 1.0, 3.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(5.0, 1.0, 0.0)
            ).inverse()),
        
            CSGNode::Sphere(Mat4::from_scale_rotation_translation(
                vec3(3.0, 3.0, 1.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(0.0, 0.0, 0.0)
            ).inverse()),
        ];
         
        /*
        self.nodes = vec![
            CSGNode::Union(1, 2, Material::None),

            CSGNode::Box(Mat4::from_scale_rotation_translation(
                vec3(2.0, 5.0, 7.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(5.0, 0.0, 0.0)
            ).inverse()),

            CSGNode::Sphere(Mat4::from_scale_rotation_translation(
                vec3(2.0, 1.0, 3.0),
                Quat::from_euler(EulerRot::XYZ, 0.0,0.0,0.0),
                vec3(5.0, 1.0, 0.0)
            ).inverse()),
        ];
        
         */
    }
    
    pub fn make_data(&mut self) {
        let (data, _) = self.add_data(0, vec![]);
        self.data = data;
    }
    
    fn add_data(&self, index: usize, mut data: Vec<u32>) -> (Vec<u32>, u32) {
        let node = &self.nodes[index];
        
        let node_data = match node {
            CSGNode::Union(child1, child2, mat) 
                | CSGNode::Remove(child1, child2, mat) 
                | CSGNode::Intersect(child1, child2, mat)
            => {
                
                let index = data.len();
                data.push(0);
                data.push(0);

                (data, data[index]) = self.add_data(*child1, data);
                (data, data[index + 1]) = self.add_data(*child2, data);
                
                let t = match node {
                    CSGNode::Union(_, _, _) => { CGS_CHILD_TYPE_UNION }
                    CSGNode::Remove(_, _, _) => { CGS_CHILD_TYPE_REMOVE }
                    CSGNode::Intersect(_, _, _) => { CGS_CHILD_TYPE_INTERSECT }
                    _ => unreachable!()
                };
                Self::node_data(index, t, *mat)
            }
            CSGNode::Box(transform) | CSGNode::Sphere(transform) => {
                let index = data.len();

                let t = match node {
                    CSGNode::Box(_) => {CGS_TYPE_BOX}
                    CSGNode::Sphere(_) => {CGS_TYPE_SPHERE}
                    _ => unreachable!()
                };
                
                data.extend_from_slice(any_as_u32_slice(transform));
                data[index + 15] = t;
                
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