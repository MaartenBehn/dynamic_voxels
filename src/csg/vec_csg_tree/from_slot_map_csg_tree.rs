use std::usize;


use crate::{csg::slot_map_csg_tree::tree::{SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey}, voxel::renderer::palette::MATERIAL_ID_NONE};

use super::tree::{VecCSGNode, VecCSGNodeData, VecCSGTree, CSG_PARENT_NONE};


impl<T: Default + Copy> From<SlotMapCSGTree<T>> for VecCSGTree<T> {
    fn from(value: SlotMapCSGTree<T>) -> Self {
        let mut tree = VecCSGTree::default();

        value.convert_node(value.root_node, &mut tree, CSG_PARENT_NONE);

        tree
    }
}

impl<T: Default + Copy> SlotMapCSGTree<T> {
    fn convert_node(&self, index: SlotMapCSGTreeKey, tree: &mut VecCSGTree<T>, parent: usize) -> usize {
        let node = self.nodes.get(index).unwrap();

        let new_node = VecCSGNode {
            data: VecCSGNodeData::All(T::default()),
            aabb: node.aabb,
            parent: parent,
        };

        let index = tree.nodes.len();
        tree.nodes.push(new_node);

        let data = match &node.data {
            SlotMapCSGNodeData::Union(c1, c2) => {
                let new_c1 = self.convert_node(*c1, tree, index);
                let new_c2 = self.convert_node(*c2, tree, index);

                VecCSGNodeData::Union(new_c1, new_c2)
            },
            SlotMapCSGNodeData::Remove(c1, c2) => {
                let new_c1 = self.convert_node(*c1, tree, index);
                let new_c2 = self.convert_node(*c2, tree, index);

                VecCSGNodeData::Remove(new_c1, new_c2)
            },
            SlotMapCSGNodeData::Intersect(c1, c2) => {
                let new_c1 = self.convert_node(*c1, tree, index);
                let new_c2 = self.convert_node(*c2, tree, index);

                VecCSGNodeData::Intersect(new_c1, new_c2)
            },
            SlotMapCSGNodeData::Mat(mat, c) => {
                let new_c = self.convert_node(*c, tree, index);

                VecCSGNodeData::Mat(*mat, new_c)
            },
            SlotMapCSGNodeData::Box(mat, material) => VecCSGNodeData::Box(*mat, *material),
            SlotMapCSGNodeData::Sphere(mat, material) => VecCSGNodeData::Sphere(*mat, *material),
            SlotMapCSGNodeData::All(material) => VecCSGNodeData::All(*material),
            SlotMapCSGNodeData::VoxelGrid(grid, offset) => VecCSGNodeData::VoxelGrid(grid.to_owned(), *offset),
        };

        tree.nodes.get_mut(index).unwrap().data = data;

        index
    }
}
