pub mod renderer;

use bvh::{aabb::{Aabb, Bounded}, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh};
use octa_force::glam::Mat4;
use slotmap::{new_key_type, SlotMap};
use crate::{aabb::AABB, voxel_tree64::VoxelTree64};

new_key_type! { pub struct SceneObjectKey; }

pub struct Scene {
    objects: Vec<SceneObject>,
    bvh: Bvh<f32, 3>,
}

enum SceneObject {
    Tree64(Tree64SceneObject),
}

pub struct Tree64SceneObject {
    mat: Mat4,
    tree: VoxelTree64,
    bvh_index: usize,
}

impl Scene {
    pub fn new() -> Self {
        Scene { 
            objects: vec![],
            bvh: Bvh::build::<SceneObject>(&mut[]) 
        }
    }
    pub fn from_tree64s(trees: Vec<Tree64SceneObject>) -> Self {
        let mut objects = trees.into_iter()
            .map(|t| SceneObject::Tree64(t))
            .collect::<Vec<_>>();

        Scene {
            bvh: Bvh::build_par(&mut objects), 
            objects,
        }
    }
}

impl Tree64SceneObject {
    pub fn new(tree: VoxelTree64, mat: Mat4) -> Self {
        Self {
            mat,
            tree,
            bvh_index: 0,
        }
    }
}

impl Bounded<f32,3> for SceneObject {
    fn aabb(&self) -> Aabb<f32,3> {
        match self {
            SceneObject::Tree64(tree64_scene_object) => {
                let aabb = AABB::from_box(&tree64_scene_object.mat, 0.0);
                aabb.into()
            },
        }
    }
}

impl BHShape<f32,3> for SceneObject {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            SceneObject::Tree64(tree64_scene_object) => tree64_scene_object.bvh_index = index,
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            SceneObject::Tree64(tree64_scene_object) => tree64_scene_object.bvh_index,
        }
    }
}
