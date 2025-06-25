use crate::vec_csg_tree::tree::{VecCSGNodeData, VecCSGTree};

use super::tree::{FastPosQueryCSGNodeData, FastPosQueryCSGTree};

impl From<VecCSGTree> for FastPosQueryCSGTree {
    fn from(value: VecCSGTree) -> Self {
        let nodes = value.nodes.iter()
            .map(|n| {
                match &n.data {
                    VecCSGNodeData::Union(a, b) => FastPosQueryCSGNodeData::Union(*a, *b),
                    VecCSGNodeData::Remove(a, b) => FastPosQueryCSGNodeData::Remove(*a, *b),
                    VecCSGNodeData::Intersect(a, b) => FastPosQueryCSGNodeData::Intersect(*a, *b),
                    VecCSGNodeData::Box(mat4, _) => FastPosQueryCSGNodeData::Box(mat4.inverse()),
                    VecCSGNodeData::Sphere(mat4, _) => FastPosQueryCSGNodeData::Sphere(mat4.inverse()),
                    VecCSGNodeData::All(_) => FastPosQueryCSGNodeData::All,
                    VecCSGNodeData::Mat(mat4, _) => panic!("Cant convert VecCSGTree with Mat Node to FastPosQueryCSGTree"),
                    VecCSGNodeData::VoxelGrid(voxel_grid, ivec3) => panic!("Cant convert VecCSGTree with VoxelGrid Node to FastPosQueryCSGTree"),
                }
            })
            .collect();

        let mut tree = FastPosQueryCSGTree { 
            nodes,
            aabb: value.nodes[0].aabb,
        };

        tree
    }
}


