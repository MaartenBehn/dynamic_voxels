use slotmap::{Key, SlotMap};

use crate::{csg_renderer::color_controller::MATERIAL_NONE, vec_csg_tree::tree::{VecCSGNodeData, VecCSGTree}};

use super::tree::{SlotMapCSGNode, SlotMapCSGNodeData, SlotMapCSGTree, SlotMapCSGTreeKey};


impl From<VecCSGTree> for SlotMapCSGTree {
    fn from(value: VecCSGTree) -> Self {
        let nodes = SlotMap::with_capacity_and_key(value.nodes.len());
        let mut tree = SlotMapCSGTree { 
            nodes,
            root_node: SlotMapCSGTreeKey::null()
        };

        tree.root_node = value.convert_node(0, &mut tree, SlotMapCSGTreeKey::null());

        tree
    }
}

impl VecCSGTree {
    fn convert_node(&self, index: usize, tree: &mut SlotMapCSGTree, parent: SlotMapCSGTreeKey) -> SlotMapCSGTreeKey {
        let node = &self.nodes[index];

        let new_node = SlotMapCSGNode {
            data: SlotMapCSGNodeData::All(MATERIAL_NONE),
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
            VecCSGNodeData::Mat(mat, c) => {
                let new_c = self.convert_node(*c, tree, index);

                SlotMapCSGNodeData::Mat(*mat, new_c)
            },
            VecCSGNodeData::Box(mat, material) => SlotMapCSGNodeData::Box(*mat, *material),
            VecCSGNodeData::Sphere(mat, material) => SlotMapCSGNodeData::Sphere(*mat, *material),
            VecCSGNodeData::All(material) => SlotMapCSGNodeData::All(*material),
            VecCSGNodeData::VoxelGrid(grid, offset) => SlotMapCSGNodeData::VoxelGrid(grid.to_owned(), *offset),
        };

        tree.nodes.get_mut(index).unwrap().data = data;

        index
    }
}
