use octa_force::glam::Vec3A;

use crate::{csg::csg_tree::tree::CSGTree, gi::gi_pool::{GI, GI_PROBE_INDEX_NONE, GIPool}, voxel::palette::palette::MATERIAL_ID_DEBUG};

pub struct GIPoolVolume {
    csg: CSGTree<u8, Vec3A, f32, 3>,
}

#[derive(Debug, Clone, Copy)]
pub struct GINone();

impl GIPoolVolume {
    pub fn new(gi_pool: &GIPool, probe_radius: f32) -> Self {
        
        
        for level_probes in gi_pool.pools.iter() {
            for probe_index in level_probes.debug_used_indices() {
                let probe = level_probes.get(probe_index);
                csg_children.push(csg.add_sphere(probe.position.as_vec3a(), probe_radius, MATERIAL_ID_DEBUG));
            }
        }
        let union_index = csg.add_union_node(csg_children);
        csg.root = union_index;

        Self { 
            csg
        }
    }
}

impl GI for GINone {
    fn new_probe_index(&self, offset: octa_force::glam::IVec3, level: u8, pop_mask: u64, children: &[crate::voxel::dag64::node::VoxelDAG64Node]) -> u32 {
        GI_PROBE_INDEX_NONE
    }
}



