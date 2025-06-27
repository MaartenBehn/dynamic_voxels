use octa_force::{vulkan::Buffer, OctaResult};

use super::{allocated_vec::{AllocatedVec, AllocatedVecIndex}, buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}};

pub struct CopyAllocatedVec<T> {
    allocation: Option<BuddyAllocation>,
    data: Vec<T>,
}

impl<T: Copy> AllocatedVec<T> for CopyAllocatedVec<T> {
    fn push(&mut self, values: &[T], allocator: &mut BuddyBufferAllocator, _buffer: &mut Buffer) -> OctaResult<Vec<AllocatedVecIndex>> {
        let index = self.data.len();
        self.data.extend_from_slice(values);

        let size = self.data.len() * size_of::<T>();
        if self.allocation.is_none() {
            self.allocation = Some(allocator.alloc(size)?);
            return Ok((index..(index + values.len())).collect());
        }
        
        let allocation = self.allocation.as_ref().unwrap();
        if allocation.size < size {
            self.allocation = Some(allocator.alloc(size)?);
        }

        Ok((index..(index + values.len())).collect())
    }

    fn remove(&mut self, indecies: &[AllocatedVecIndex], _allocator: &mut BuddyBufferAllocator, _buffer: &mut Buffer) {
        for i in indecies {
            self.data.swap_remove(*i);
        }
    }

    fn flush(&mut self, buffer: &mut Buffer) -> OctaResult<()> {
        buffer.copy_data_to_buffer_without_aligment(&self.data, self.allocation.as_ref().unwrap().start)
    }
}

impl<T> Default for CopyAllocatedVec<T> {
    fn default() -> Self {
        Self {
            allocation: None,
            data: vec![],
        }
    }
}
