use octa_force::glam::{IVec3, Mat4};
use slotmap::{new_key_type, Key, SlotMap};

use crate::{aabb::AABB, csg_renderer::color_controller::Material, model_synthesis::builder::BU, voxel::grid::VoxelGrid};

new_key_type! { pub struct SlotMapCSGTreeKey; }

impl BU for SlotMapCSGTreeKey {}

#[derive(Clone, Debug)]
pub struct SlotMapCSGTree {
    pub nodes: SlotMap<SlotMapCSGTreeKey, SlotMapCSGNode>,
    pub root_node: SlotMapCSGTreeKey,
}

#[derive(Clone, Debug)]
pub struct SlotMapCSGNode {
    pub data: SlotMapCSGNodeData,
    pub aabb: AABB,
    pub parent: SlotMapCSGTreeKey,
}

#[derive(Clone, Debug)]
pub enum SlotMapCSGNodeData {
    Union(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Remove(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Intersect(SlotMapCSGTreeKey, SlotMapCSGTreeKey),
    Mat(Mat4, SlotMapCSGTreeKey),
    Box(Mat4, Material),
    Sphere(Mat4, Material),
    All(Material),
    VoxelGrid(VoxelGrid, IVec3),
}

impl SlotMapCSGNode {
    pub fn new(data: SlotMapCSGNodeData) -> Self {
        SlotMapCSGNode {
            data,
            aabb: Default::default(),
            parent: SlotMapCSGTreeKey::null(),
        }
    }
}

impl SlotMapCSGTree {
    pub fn is_empty(&self) -> bool {
        self.root_node == SlotMapCSGTreeKey::null()
    }

    pub fn from_node(node: SlotMapCSGNode) -> Self {
        let mut tree = Self {
            nodes: SlotMap::with_capacity_and_key(1),
            root_node: SlotMapCSGTreeKey::null(),
        };
        let index = tree.nodes.insert(node);
        tree.root_node = index;

        tree
    }
}
