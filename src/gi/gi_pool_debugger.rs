use octa_force::glam::Vec3A;

use crate::{csg::csg_tree::tree::CSGTree, gi::gi_pool::{GI, GI_PROBE_INDEX_NONE, GIPool}, voxel::palette::palette::MATERIAL_ID_DEBUG};

#[derive(Debug, Clone, Copy)]
pub struct GINone;

impl GI for GINone {
    fn new_probe_index(&self, offset: octa_force::glam::IVec3, level: u8, pop_mask: u64, children: &[crate::voxel::dag64::node::VoxelDAG64Node]) -> u32 {
        GI_PROBE_INDEX_NONE
    }
}



