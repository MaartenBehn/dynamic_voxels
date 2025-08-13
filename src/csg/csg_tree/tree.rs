use octa_force::glam::{IVec3, Mat4};
use slotmap::{new_key_type, Key, SlotMap};
use smallvec::SmallVec;

use crate::{csg::{all::CSGAll, r#box::CSGBox, sphere::CSGSphere}, model::generation::traits::BU, util::{aabb3d::AABB, iaabb3d::AABBI}, voxel::grid::{offset::OffsetVoxelGrid, shared::SharedVoxelGrid, VoxelGrid}};

new_key_type! { pub struct CSGTreeKey; }

impl BU for CSGTreeKey {}

#[derive(Clone, Debug, Default)]
pub struct CSGTree<T> {
    pub nodes: SlotMap<CSGTreeKey, CSGNode<T>>,
    pub root_node: CSGTreeKey,
}

#[derive(Clone, Debug)]
pub struct CSGNode<T> {
    pub data: CSGNodeData<T>,
    pub aabb: AABB,
    pub aabbi: AABBI,
    pub parent: CSGTreeKey,
}

#[derive(Clone, Debug)]
pub enum CSGNodeData<T> {
    Union(CSGTreeKey, CSGTreeKey),
    Remove(CSGTreeKey, CSGTreeKey),
    Intersect(CSGTreeKey, CSGTreeKey),
    Box(CSGBox<T>), // Inverse Mat
    Sphere(CSGSphere<T>), // Inverse Mat
    All(CSGAll<T>),
    OffsetVoxelGrid(OffsetVoxelGrid),
    SharedVoxelGrid(SharedVoxelGrid),
}

impl<T> CSGNode<T> {
    pub fn new(data: CSGNodeData<T>) -> Self {
        CSGNode {
            data,
            aabb: Default::default(),
            aabbi: Default::default(),
            parent: CSGTreeKey::null(),
        }
    }

    pub fn set_aabb(&mut self, aabb: AABB) {
        self.aabb = aabb;
        self.aabbi = aabb.into();
    }
}

impl<T: Clone> CSGTree<T> {
    pub fn is_empty(&self) -> bool {
        self.root_node == CSGTreeKey::null()
    }

    pub fn from_node(node: CSGNode<T>) -> Self {
        let mut tree = Self {
            nodes: SlotMap::with_capacity_and_key(1),
            root_node: CSGTreeKey::null(),
        };
        let index = tree.nodes.insert(node);
        tree.root_node = index;

        tree.set_all_aabbs();

        tree
    }
}
