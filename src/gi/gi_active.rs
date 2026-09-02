use std::sync::{Arc, atomic::AtomicUsize};

use octa_force::{OctaResult, glam::{IVec3, Vec3}, log::info};

use crate::{gi::gi_pool::GIPool, scene::staging_copies::SceneStagingBuilder, util::{buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}, math::to_mb, shader_constants::GI_ATLAS_SIZE}};

pub type ActiveProbeIndex = u16; 
pub const ACTIVE_PROBE_INDEX_NONE: ActiveProbeIndex = ActiveProbeIndex::MAX;
pub const NUM_ACTIVE_PROBES: usize = GI_ATLAS_SIZE * GI_ATLAS_SIZE; 
pub const INITAL_MAX_PROBES: usize = u16::MAX as usize;

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
       
        let probe_map_size = INITAL_MAX_PROBES * size_of::<ActiveProbeIndex>(); 
        info!("Probe Map Buffer size: {:.04} MB", to_mb(probe_map_size));
        
        let probe_map_alloc = allocator.alloc(probe_map_size)?;

        let probe_data_size = NUM_ACTIVE_PROBES * size_of::<ActiveProbeIndex>(); 
        info!("Active Probe Data Buffer size: {:.04} MB", to_mb(probe_data_size));

        let probe_data_alloc = allocator.alloc(probe_data_size)?;

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

        self.active_size = 0;
        for (key, probe) in pool.probes.unique_iter_with_idx() {
            if self.active_size as usize >= NUM_ACTIVE_PROBES {
                break;
            }

            let active_probe = ActiveProbeData {
                position: probe.position,
                start_index: probe.start_index,
                object_offset: probe.object_offset,
            };

            builder.push(
                &[active_probe], 
                self.probe_data_alloc.start() + self.active_size as usize * size_of::<ActiveProbeData>());
            
            builder.push(
                &[self.active_size as ActiveProbeIndex],
                self.probe_map_alloc.start() + key * size_of::<ActiveProbeIndex>());

            self.active_size += 1;
        }
    }
}
