use std::sync::{Arc, atomic::AtomicUsize};

use octa_force::{OctaResult, glam::{IVec3, Vec3}};

use crate::{gi::gi_pool::GIPool, scene::staging_copies::SceneStagingBuilder, util::{buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, shader_constants::GI_ATLAS_SIZE}};

pub type ActiveProbeIndex = u16; 
pub const ACTIVE_PROBE_INDEX_NONE: ActiveProbeIndex = ActiveProbeIndex::MAX;
pub const NUM_ACTIVE_PROBES: usize = GI_ATLAS_SIZE * GI_ATLAS_SIZE; 
pub const INITAL_MAX_PROBES: usize = 100000;

#[derive(Debug)]
pub struct GIActive {
    pub probe_map_alloc: ManualBuddyAllocation,
    pub probe_data_alloc: ManualBuddyAllocation,
    pub active_size: u32,

    pub write_initial: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ActiveProbeData {
    pub position: Vec3,
    pub start_index: u32,
    pub object_offset: u32,
}

impl GIActive {
    pub fn new(allocator: &mut BuddyAllocator) -> OctaResult<Self> {
        debug_assert!(ActiveProbeIndex::MAX as usize > NUM_ACTIVE_PROBES);
        
        let probe_map_alloc = allocator.alloc(
            INITAL_MAX_PROBES * size_of::<ActiveProbeIndex>())?;
        
        let probe_data_alloc = allocator.alloc(
            NUM_ACTIVE_PROBES * size_of::<ActiveProbeData>())?;

        Ok(Self {
            probe_map_alloc,
            probe_data_alloc,
            active_size: 0,
            write_initial: true,
        })
    }

    pub fn update(&mut self, pool: &mut GIPool, builder: &mut SceneStagingBuilder) {
        if self.write_initial {
            builder.push(
                &vec![ACTIVE_PROBE_INDEX_NONE; self.probe_map_alloc.size()], 
                self.probe_map_alloc.start());

            self.write_initial = false;
        }

        for (i, probe) in pool.pools[0].unique_iter().enumerate() {
            if i >= NUM_ACTIVE_PROBES {
                break;
            }

            let active_probe = ActiveProbeData {
                position: probe.position,
                start_index: probe.start_index,
                object_offset: probe.object_offset,
            };

            builder.push(
                &[active_probe], 
                self.probe_data_alloc.start() + i * size_of::<ActiveProbeData>());
            
            builder.push(
                &[i as ActiveProbeIndex],
                self.probe_map_alloc.start() + i * size_of::<ActiveProbeIndex>());

            self.active_size += 1;
        }
    }
}
