use octa_force::{anyhow::bail, glam::{uvec3, vec4, Vec4, Vec4Swizzles}, OctaResult};


use crate::{csg::csg_tree::tree::CSGTree, util::math::to_1d, volume::{VolumeQureyPosValid, VolumeQureyPosValueI}};

use super::VoxelGrid;

impl TryFrom<CSGTree<u8>> for VoxelGrid {
    type Error = octa_force::anyhow::Error;
    
    fn try_from(value: CSGTree<u8>) -> OctaResult<Self> {
        let root = &value.nodes[value.root_node]; 
        
        if !root.aabb.min.is_finite() || !root.aabb.max.is_finite() {
            bail!("Can only transform finite csg trees.");             
        }

        let mut grid = VoxelGrid::empty(root.aabbi.size().as_uvec3());
        
        for x in 0..grid.size.x {
            for y in 0..grid.size.y {
                for z in 0..grid.size.z {
                    let pos = uvec3(x, y, z);
                    let index = to_1d(pos, grid.size);
                    let in_csg_pos = root.aabbi.min + pos.as_ivec3();

                    grid.data[index] = value.get_value_i(in_csg_pos);
                }
            }
        } 
        
        Ok(grid)
    }
}

