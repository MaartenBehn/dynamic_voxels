use octa_force::glam::{IVec3, Mat4};
use slotmap::{new_key_type, Key, SlotMap};

use crate::{model::generation::traits::BU, util::aabb3d::AABB, voxel::{grid::VoxelGrid, renderer::palette::Material}};

new_key_type! { pub struct SlotMapCSGTreeKey; }

impl BU for SlotMapCSGTreeKey {}

#[derive(Clone, Debug, Default)]
pub struct SlotMapCSGTree<T> {
    pub nodes: SlotMap<SlotMapCSGTreeKey, SlotMapCSGNode<T>>,
    pub root_node: SlotMapCSGTreeKey,
}

#[derive(Clone, Debug)]
pub struct SlotMapCSGNode<T> {
    pub data: SlotMapCSGNodeData<T>,
    pub aabb: AABB,
    pub parent: SlotMapCSGTreeKey,
}

#[derive(Clone, Debug)]
pub enum SlotMapCSGNodeData<T> {
    Union(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Remove(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Intersect(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Mat(Mat4, SlotMapCSGTreeKey),
    Box(Mat4, T), // Inverse Mat
    Sphere(Mat4, T), // Inverse Mat
    All(T),
    VoxelGrid(VoxelGrid, IVec3),
}

impl<T> SlotMapCSGNode<T> {
    pub fn new(data: SlotMapCSGNodeData<T>) -> Self {
        SlotMapCSGNode {
            data,
            aabb: Default::default(),
            parent: SlotMapCSGTreeKey::null(),
        }
    }
}

impl<T: Clone> SlotMapCSGTree<T> {
    pub fn is_empty(&self) -> bool {
        self.root_node == SlotMapCSGTreeKey::null()
    }

    pub fn from_node(node: SlotMapCSGNode<T>) -> Self {
        let mut tree = Self {
            nodes: SlotMap::with_capacity_and_key(1),
            root_node: SlotMapCSGTreeKey::null(),
        };
        let index = tree.nodes.insert(node);
        tree.root_node = index;

        tree.set_all_aabbs();

        tree
    }
}
