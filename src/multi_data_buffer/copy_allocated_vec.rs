use octa_force::OctaResult;

use super::{allocated_vec::{AllocatedVec, AllocatedVecIndex}, buddy_controller::BuddyBufferAllocator};

pub struct CopyAllocatedVec<T> {
    allocation: Option<(usize, usize)>,
    data: Vec<T>,
}

impl<T: Clone> AllocatedVec<T> for CopyAllocatedVec<T> {
    fn push(&mut self, values: &[T], allocator: &mut BuddyBufferAllocator) -> OctaResult<Vec<AllocatedVecIndex>> {
        let index = self.data.len();
        self.data.extend_from_slice(values);

        let size = self.data.len() * size_of::<T>();
        if self.allocation.is_none() {
            self.allocation = Some(allocator.alloc(size)?);
            return Ok((index..(index + values.len())).collect());
        }
        
        let allocation = self.allocation.as_ref().unwrap();
        if allocation.1 < size {
            allocator.dealloc(allocation.0)?; 
            self.allocation = Some(allocator.alloc(size)?);
        }

        Ok((index..(index + values.len())).collect())
    }

    fn remove(&mut self, indecies: &[AllocatedVecIndex], allocator: &mut BuddyBufferAllocator) {
        for i in indecies {
            self.data.swap_remove(*i);
        }
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
