use std::sync::{Arc, atomic::AtomicUsize};

use octa_force::{OctaResult, glam::{IVec3, Vec3}};

use crate::{gi::gi_pool::GIPool, scene::staging_copies::SceneStagingBuilder, util::buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, voxel::renderer::gi::GI_ATLAS_SIZE};

pub const ACTIVE_PROBE_INDEX_NONE: u32 = u32::MAX;

#[derive(Debug)]
pub struct GIActive {
    pub alloc: ManualBuddyAllocation,
    pub active_size: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ActiveProbeData {
    pub position: Vec3,
    pub start_index: u32,
    pub object_offset: u32,
}

impl GIActive {
    pub fn new(allocator: &mut BuddyAllocator) -> OctaResult<Self> {
        
        let num = GI_ATLAS_SIZE * GI_ATLAS_SIZE;
        let alloc = allocator.alloc(num * size_of::<ActiveProbeData>())?;

        Ok(Self {
            alloc,
            active_size: 0,
        })
    }

    pub fn update(&mut self, pool: &mut GIPool, builder: &mut SceneStagingBuilder) {

        for (i, probe) in pool.pools[0].unique_iter().enumerate() {
            let active_probe = ActiveProbeData {
                position: probe.position,
                start_index: probe.start_index,
                object_offset: probe.object_offset,
            };

            builder.push(&[active_probe], self.alloc.start() + i * size_of::<ActiveProbeData>());
            self.active_size += 1;
        }
    }
}
