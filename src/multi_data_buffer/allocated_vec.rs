use octa_force::OctaResult;

use super::buddy_controller::BuddyBufferAllocator;

pub type AllocatedVecIndex = usize;

pub trait AllocatedVec<T>: Default {
    fn push(&mut self, values: &[T], allocator: &mut BuddyBufferAllocator) -> OctaResult<Vec<AllocatedVecIndex>>;
    fn remove(&mut self, indecies: &[AllocatedVecIndex], allocator: &mut BuddyBufferAllocator);
}
