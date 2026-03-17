use octa_force::OctaResult;
use slotmap::{SlotMap, new_key_type};

use crate::{gi::gi_pool::GIPool, util::buddy_allocator::{BuddyAllocator, ManualBuddyAllocation}};

new_key_type! { pub struct SceneGIKey; }

#[derive(Debug)]
pub struct SceneGI {
    pub gi_pool: GIPool,
    pub alloc: ManualBuddyAllocation,
    pub needs_update: bool,
}

impl SceneGI {
    pub fn new(allocator: &mut BuddyAllocator) -> OctaResult<Self> {
       
        let gi_pool = GIPool::new(10);
        let alloc = allocator.alloc(gi_pool.get_memory_size())?;

        Ok(Self {
            gi_pool,
            alloc,
            needs_update: false,
        })
    }   
}
