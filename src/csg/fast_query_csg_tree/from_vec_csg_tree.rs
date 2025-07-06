
use crate::csg::vec_csg_tree::tree::{VecCSGNodeData, VecCSGTree};

use super::tree::{FastQueryCSGNodeData, FastQueryCSGTree};

impl From<VecCSGTree> for FastQueryCSGTree<()> {
    fn from(value: VecCSGTree) -> Self {
        let nodes = value.nodes.iter()
            .map(|n| {
                match &n.data {
                    VecCSGNodeData::Union(a, b) => FastQueryCSGNodeData::Union(*a, *b),
                    VecCSGNodeData::Remove(a, b) => FastQueryCSGNodeData::Remove(*a, *b),
                    VecCSGNodeData::Intersect(a, b) => FastQueryCSGNodeData::Intersect(*a, *b),
                    VecCSGNodeData::Box(mat4, v) => FastQueryCSGNodeData::Box(mat4.inverse(), ()),
                    VecCSGNodeData::Sphere(mat4, _) => FastQueryCSGNodeData::Sphere(mat4.inverse(), ()),
                    VecCSGNodeData::All(_) => FastQueryCSGNodeData::All(()),
                    VecCSGNodeData::Mat(mat4, _) => panic!("Cant convert VecCSGTree with Mat Node to FastPosQueryCSGTree"),
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

impl From<VecCSGTree> for FastQueryCSGTree<u8> {
    fn from(value: VecCSGTree) -> Self {
        let nodes = value.nodes.iter()
            .map(|n| {
                match &n.data {
                    VecCSGNodeData::Union(a, b) => FastQueryCSGNodeData::Union(*a, *b),
                    VecCSGNodeData::Remove(a, b) => FastQueryCSGNodeData::Remove(*a, *b),
                    VecCSGNodeData::Intersect(a, b) => FastQueryCSGNodeData::Intersect(*a, *b),
                    VecCSGNodeData::Box(mat4, v) => FastQueryCSGNodeData::Box(mat4.inverse(), *v),
                    VecCSGNodeData::Sphere(mat4, v) => FastQueryCSGNodeData::Sphere(mat4.inverse(), *v),
                    VecCSGNodeData::All(v) => FastQueryCSGNodeData::All(*v),
                    VecCSGNodeData::Mat(mat4, v) => panic!("Cant convert VecCSGTree with Mat Node to FastPosQueryCSGTree"),
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


