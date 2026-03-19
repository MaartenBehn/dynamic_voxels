use octa_force::OctaResult;
use slotmap::{SlotMap, new_key_type};

use crate::{gi::{gi_active::GIActive, gi_pool::GIPool}, scene::staging_copies::SceneStagingBuilder, util::buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}};

new_key_type! { pub struct SceneGIKey; }

#[derive(Debug)]
pub struct SceneGI {
    pub gi_pool: GIPool,
    pub active: GIActive,
    pub needs_update: bool,
}

impl SceneGI {
    pub fn new(allocator: &mut BuddyAllocator) -> OctaResult<Self> {
       
        let gi_pool = GIPool::new(10);
        let active = GIActive::new(allocator)?;

        Ok(Self {
            gi_pool,
            active,
            needs_update: false,
        })
    }  

    pub fn update(&mut self, builder: &mut SceneStagingBuilder) {
        if !self.needs_update {
            return;
        }

        self.active.update(&mut self.gi_pool, builder);
        self.needs_update = false;
    }
}
