

use crate::csg::csg_tree::tree::{CSGNodeData, CSGTree};

use super::tree::{FastQueryCSGNodeData, FastQueryCSGTree};

impl<T: Clone + std::fmt::Debug> From<CSGTree<T>> for FastQueryCSGTree<T> {
    fn from(mut value: CSGTree<T>) -> Self {
        value.set_all_aabbs();

        let nodes = value.nodes.values()
            .map(|n| {
                match &n.data {
                    CSGNodeData::Union(a, b) => 
                        FastQueryCSGNodeData::Union(value.get_value_index_of_key(*a), value.get_value_index_of_key(*a)),

                    CSGNodeData::Remove(a, b) => 
                        FastQueryCSGNodeData::Remove(value.get_value_index_of_key(*a), value.get_value_index_of_key(*b)),

                    CSGNodeData::Intersect(a, b) => 
                        FastQueryCSGNodeData::Intersect(value.get_value_index_of_key(*a), value.get_value_index_of_key(*b)),

                    CSGNodeData::Box(mat, v) => FastQueryCSGNodeData::Box(*mat, v.to_owned()),

                    CSGNodeData::Sphere(mat, v) => FastQueryCSGNodeData::Sphere(*mat, v.to_owned()),

                    CSGNodeData::All(v) => FastQueryCSGNodeData::All(v.to_owned()),

                    CSGNodeData::OffsetVoxelGrid(offset_voxel_grid) => todo!(),
                    CSGNodeData::SharedVoxelGrid(shared_voxel_grid) => todo!(),
                }
            })
            .collect();

        let mut tree = FastQueryCSGTree { 
            nodes,
            aabb: value.nodes[value.root_node].aabb,
            aabbi: value.nodes[value.root_node].aabbi,
            root: value.get_value_index_of_key(value.root_node),
        };

        tree
    }
}


