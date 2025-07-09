use octa_force::{anyhow::bail, glam::{uvec3, vec4, Vec4, Vec4Swizzles}, OctaResult};


use crate::{csg::vec_csg_tree::tree::VecCSGTree, util::math::to_1d, volume::VolumeQureyPosValid, voxel::renderer::palette::{MATERIAL_ID_BASE, MATERIAL_ID_NONE}};

use super::VoxelGrid;

impl TryFrom<VecCSGTree<u8>> for VoxelGrid {
    type Error = octa_force::anyhow::Error;
    
    fn try_from(value: VecCSGTree<u8>) -> OctaResult<Self> {
        let root = &value.nodes[0]; 
        
        if !root.aabb.min.is_finite() || !root.aabb.max.is_finite() {
            bail!("Can only transform finite csg trees.");             
        }

        let mut grid = VoxelGrid::new(root.aabb.size().xyz().as_uvec3());
        
        for x in 0..grid.size.x {
            for y in 0..grid.size.y {
                for z in 0..grid.size.z {
                    let pos = uvec3(x as u32, y as u32, z as u32);
                    let index = to_1d(pos, grid.size);
                    let in_csg_pos = root.aabb.min.xyz() + pos.as_vec3();
                    let filled = value.is_position_valid_vec3(Vec4::from((in_csg_pos, 1.0)));

                    grid.data[index] = if filled { MATERIAL_ID_BASE } else { MATERIAL_ID_NONE };
                }
            }
        } 
        
        Ok(grid)
    }
}

