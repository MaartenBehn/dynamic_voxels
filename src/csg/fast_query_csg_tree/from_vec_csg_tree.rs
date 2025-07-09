
use crate::csg::vec_csg_tree::tree::{VecCSGNodeData, VecCSGTree};

use super::tree::{FastQueryCSGNodeData, FastQueryCSGTree};

impl<T: Clone> From<VecCSGTree<T>> for FastQueryCSGTree<T> {
    fn from(value: VecCSGTree<T>) -> Self {
        let nodes = value.nodes.iter()
            .map(|n| {
                match &n.data {
                    VecCSGNodeData::Union(a, b) => FastQueryCSGNodeData::Union(*a, *b),
                    VecCSGNodeData::Remove(a, b) => FastQueryCSGNodeData::Remove(*a, *b),
                    VecCSGNodeData::Intersect(a, b) => FastQueryCSGNodeData::Intersect(*a, *b),
                    VecCSGNodeData::Box(mat4, v) => FastQueryCSGNodeData::Box(mat4.inverse(), v.to_owned()),
                    VecCSGNodeData::Sphere(mat4, v) => FastQueryCSGNodeData::Sphere(mat4.inverse(), v.to_owned()),
                    VecCSGNodeData::All(v) => FastQueryCSGNodeData::All(v.to_owned()),
                    VecCSGNodeData::Mat(mat, c) => FastQueryCSGNodeData::Mat(mat.to_owned(), c.to_owned()),
                    VecCSGNodeData::VoxelGrid(voxel_grid, ivec3) => panic!("Cant convert VecCSGTree with VoxelGrid Node to FastPosQueryCSGTree"),
                }
            })
            .collect();

        let mut tree = FastQueryCSGTree { 
            nodes,
            aabb: value.nodes[0].aabb,
        };

        tree
    }
}


