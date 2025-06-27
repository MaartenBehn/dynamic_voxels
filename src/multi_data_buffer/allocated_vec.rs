use octa_force::{vulkan::Buffer, OctaResult};

use super::buddy_buffer_allocator::BuddyBufferAllocator;


pub type AllocatedVecIndex = usize;

pub trait AllocatedVec<T>: Default {
    fn push(&mut self, values: &[T], allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<Vec<AllocatedVecIndex>>;
    fn remove(&mut self, indecies: &[AllocatedVecIndex], allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer);
    fn flush(&mut self, buffer: &mut Buffer) -> OctaResult<()>;
}
