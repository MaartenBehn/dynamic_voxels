use slotmap::{Key, SlotMap};

use crate::{csg::vec_csg_tree::tree::{VecCSGNodeData, VecCSGTree}, voxel::{grid::offset::OffsetVoxelGrid, renderer::palette::MATERIAL_ID_NONE}};

use super::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};


impl<T: Default + Copy> From<VecCSGTree<T>> for SlotMapCSGTree<T> {
    fn from(value: VecCSGTree<T>) -> Self {
        let nodes = SlotMap::with_capacity_and_key(value.nodes.len());
        let mut tree = SlotMapCSGTree::<T> { 
            nodes,
            root_node: SlotMapCSGTreeKey::null()
        };

        tree.root_node = value.convert_node(0, &mut tree, SlotMapCSGTreeKey::null());

        tree
    }
}

impl<T: Default + Copy> VecCSGTree<T> {
    fn convert_node(&self, index: usize, tree: &mut SlotMapCSGTree<T>, parent: SlotMapCSGTreeKey) -> SlotMapCSGTreeKey {
        let node = &self.nodes[index];

        let new_node = SlotMapCSGNode {
            data: SlotMapCSGNodeData::All(T::default()),
            aabb: node.aabb,
            parent: parent,
        };

        let index = tree.nodes.insert(new_node);

        let data = match &node.data {
            VecCSGNodeData::Union(c1, c2) => {
                let new_c1 = self.convert_node(*c1, tree, index);
                let new_c2 = self.convert_node(*c2, tree, index);

                SlotMapCSGNodeData::Union(new_c1, new_c2)
            },
            VecCSGNodeData::Remove(c1, c2) => {
                let new_c1 = self.convert_node(*c1, tree, index);
                let new_c2 = self.convert_node(*c2, tree, index);

                SlotMapCSGNodeData::Remove(new_c1, new_c2)
            },
            VecCSGNodeData::Intersect(c1, c2) => {
                let new_c1 = self.convert_node(*c1, tree, index);
                let new_c2 = self.convert_node(*c2, tree, index);

                SlotMapCSGNodeData::Intersect(new_c1, new_c2)
            },
            VecCSGNodeData::Mat(mat, c) => unreachable!(),
            VecCSGNodeData::Box(mat, material) => SlotMapCSGNodeData::Box(*mat, *material),
            VecCSGNodeData::Sphere(mat, material) => SlotMapCSGNodeData::Sphere(*mat, *material),
            VecCSGNodeData::All(material) => SlotMapCSGNodeData::All(*material),
            VecCSGNodeData::VoxelGrid(grid, offset) => SlotMapCSGNodeData::OffsetVoxelGrid(OffsetVoxelGrid::from_grid(grid.to_owned(), *offset)),
        };

        tree.nodes.get_mut(index).unwrap().data = data;

        index
    }
}
