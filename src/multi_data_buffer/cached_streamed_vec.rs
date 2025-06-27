
use std::{iter, marker::PhantomData};

use octa_force::{log::error, vulkan::Buffer, OctaResult};

use super::{allocated_vec::{AllocatedVec, AllocatedVecIndex}, buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}};


#[derive(Debug)]
pub struct CachedStreamedVec<T> {
    allocations: Vec<CachedStreamedAllocation>,
    full_allocations: Vec<CachedStreamedAllocation>,
    pub minimum_allocation_size: usize,
    phantom: PhantomData<T>,
}

#[derive(Debug)]
struct CachedStreamedAllocation {
    allocation: BuddyAllocation,
    start_index: usize,
    capacity: usize,
    padding: usize,
    free_ranges: Vec<(usize, usize)>,
    needs_free_optimization: bool,
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Eq, PartialEq)]
struct CompactRange {
    start: u32,
    length: u8,
}

impl CompactRange {
    fn as_range(&self) -> std::ops::Range<usize> {
        self.start as usize..self.start as usize + self.length as usize
    }
}

impl<T: Copy + Default> AllocatedVec<T> for CachedStreamedVec<T> {
    fn push(&mut self, mut values: &[T], allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<Vec<AllocatedVecIndex>> {
        let mut res = Vec::with_capacity(values.len());
        let data = buffer.get_mapped_slice::<T>();

        // Fill exsiting allocations
        while let Some(mut alloc) = self.allocations.pop() {
            while let Some((start, end)) = alloc.free_ranges.pop() {
                let len = end - start;
                if values.len() >= len {
                    let (a, b) = values.split_at(len);
                    values = b;

                    let range = (alloc.start_index + start)..(alloc.start_index + end); 
                    data[range].copy_from_slice(a);
                    res.extend(range);

                } else {
                    let new_start = start + values.len();
                    
                    let range = (alloc.start_index + start)..(alloc.start_index + new_start); 
                    data[range].copy_from_slice(a);
                    res.extend(range);

                    alloc.free_ranges.push((new_start, end));

                    self.allocations.push(alloc);
                    return Ok(res);
                }
            }

            self.full_allocations.push(alloc);
        }

        if values.is_empty() {
            return Ok(res);
        }

        // Allocate further for the res
        let allocation = allocator.alloc(self.minimum_allocation_size.max(values.len()))?;

        let capacity = allocation.size / size_of::<T>();
        let padding = allocation.start % size_of::<T>();
        let start_index = allocation.start / size_of::<T>() + (padding != 0) as usize;

        let range = start_index..(start_index + values.len()); 
        data[range].copy_from_slice(values);
        res.extend(range);

        if values.len() < capacity {
            self.allocations.push(CachedStreamedAllocation {
                allocation,
                start_index,
                capacity, 
                padding,
                free_ranges: vec![(values.len(), capacity)], 
                needs_free_optimization: false,
            });
        } else {
            self.full_allocations.push(CachedStreamedAllocation {
                allocation,
                start_index,
                capacity,
                padding,
                free_ranges: vec![],
                needs_free_optimization: false,
            });
        }

        Ok(res)
    }

    fn remove(&mut self, indecies: &[AllocatedVecIndex], allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) {
        for index in indecies {
            let res = self.allocations.iter_mut()
                .find(|a| a.start_index <= *index && (a.start_index + a.capacity) > *index);

            if res.is_some() {
                let alloc = res.unwrap();
                let index = index - alloc.start_index;
                alloc.free_ranges.push((index, index + 1));
                alloc.needs_free_optimization = true;
                continue;
            }  

            let res = self.full_allocations.iter()
                .position(|a| a.start_index <= *index && (a.start_index + a.capacity) > *index);

            if res.is_some() {
                let mut alloc = self.full_allocations.swap_remove(res.unwrap());
                let index = index - alloc.start_index;
                alloc.free_ranges.push((index, index + 1));
                self.allocations.push(alloc);
                continue;
            }  

            error!("Allocated Index {index} not found!");
        }
    }

    fn flush(&mut self, buffer: &mut Buffer) -> OctaResult<()> { 
        Ok(())
    }

    fn optimize(&mut self) {
        for alloc in self.allocations.iter_mut() {
            alloc.optimize_free_ranges();            
        } 
    }
}

impl CachedStreamedAllocation {
    pub fn optimize_free_ranges(&mut self) {
        if !self.needs_free_optimization {
            return;
        }

        self.free_ranges.sort_by(|a, b| a.0.cmp(&b.0));
        for i in (0..self.free_ranges.len()).rev().skip(1) {
            let range = self.free_ranges[i];
            let last_range = self.free_ranges[i+1];

            if range.1 >= last_range.0 {
                self.free_ranges.swap_remove(i+1);
                self.free_ranges[i] = (range.0, last_range.1.max(range.0));
            }
        }
        self.needs_free_optimization = false;
    } 
}

impl<T> Default for CachedStreamedVec<T> {
    fn default() -> Self {
        Self { 
            allocations: vec![],
            minimum_allocation_size: 32,
            full_allocations: vec![],
            phantom: PhantomData::default(),
        }
    } 
}
