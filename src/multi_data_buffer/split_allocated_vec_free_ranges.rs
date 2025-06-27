
use std::{iter};

use free_ranges::{FreeRanges, Range};
use nalgebra::Norm;
use octa_force::{itertools::Itertools, log::error, vulkan::Buffer, OctaResult};

use super::{allocated_vec::{AllocatedVec, AllocatedVecIndex}, buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}};


#[derive(Debug)]
pub struct SplitAllocatedVec<T> {
    allocations: Vec<SplitAllocation<T>>,
    full_allocations: Vec<SplitAllocation<T>>,
    pub minimum_allocation_size: usize,
}

#[derive(Debug)]
struct SplitAllocation<T> {
    allocation: BuddyAllocation,
    start_index: usize,
    padding: usize,
    data: Vec<T>,
    free_ranges: FreeRanges,
    changed_ranges: FreeRanges,
    needs_free_optimization: bool,
}

impl<T: Copy + Default> AllocatedVec<T> for SplitAllocatedVec<T> {
    fn push(&mut self, mut values: &[T], allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<Vec<AllocatedVecIndex>> {
        let mut res = Vec::with_capacity(values.len());

        // Fill exsiting allocations
        while let Some(mut alloc) = self.allocations.pop() {
            while let Some((start, end)) = alloc.free_ranges.pop() {
                let len = end - start;
                if values.len() >= len {
                    let (a, b) = values.split_at(len);
                    values = b;
                    alloc.data[start..end].copy_from_slice(a);
                    alloc.changed_ranges.push((start, end));

                    res.extend((alloc.start_index + start)..(alloc.start_index + end));
                } else {
                    let new_start = start + values.len();
                    alloc.data[start..new_start].copy_from_slice(values);
                    alloc.free_ranges.push((new_start, end));
                    alloc.changed_ranges.push((start, new_start));
                    
                    res.extend((alloc.start_index + start)..(alloc.start_index + new_start));

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
        let mut data = Vec::with_capacity(capacity); 
        data.extend_from_slice(values);
        data[(values.len()..)].fill(T::default());

        if values.len() < capacity {
            self.allocations.push(SplitAllocation {
                allocation,
                start_index,
                padding,
                data,
                free_ranges: vec![(values.len(), capacity)], 
                changed_ranges: vec![(0, values.len())],
                needs_free_optimization: false,
            });
        } else {
            self.full_allocations.push(SplitAllocation {
                allocation,
                start_index,
                padding,
                data,
                free_ranges: vec![],
                changed_ranges: vec![(0, capacity)],
                needs_free_optimization: false,
            });
        }

        Ok(res)
    }

    fn remove(&mut self, indecies: &[AllocatedVecIndex], allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) {
        for index in indecies {
            let res = self.allocations.iter_mut()
                .find(|a| a.start_index <= *index && (a.start_index + a.data.len()) > *index);

            if res.is_some() {
                let alloc = res.unwrap();
                let index = index - alloc.start_index;
                alloc.free_ranges.push((index, index + 1));
                alloc.needs_free_optimization = true;
                continue;
            }  

            let res = self.full_allocations.iter()
                .position(|a| a.start_index <= *index && (a.start_index + a.data.len()) > *index);

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
        for alloc in self.allocations.iter_mut() {
            alloc.optimize_changed_ranges();

            for (start, end) in alloc.changed_ranges.iter() {
                buffer.copy_data_to_buffer_without_aligment(&alloc.data[*start..*end], alloc.allocation.start + start + alloc.padding)?;
            }
        }

        Ok(())
    }
}

impl<T> SplitAllocatedVec<T> {
    pub fn optimize_free_ranges(&mut self) {
        for alloc in self.allocations.iter_mut() {
            alloc.optimize_free_ranges();            
        }
    }

    pub fn optimize_changed_ranges(&mut self) {
        for alloc in self.allocations.iter_mut() { 
            alloc.optimize_changed_ranges();
        }
    }
}

impl<T> SplitAllocation<T> {
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

    pub fn optimize_changed_ranges(&mut self) {
        self.changed_ranges.sort_by(|a, b| a.0.cmp(&b.0));
        for i in (0..self.changed_ranges.len()).rev().skip(1) {
            let range = self.changed_ranges[i];
            let last_range = self.changed_ranges[i+1];

            if range.1 >= last_range.0 {
                self.changed_ranges.swap_remove(i+1);
                self.changed_ranges[i] = (range.0, last_range.1.max(range.0));
            }
        }
    }

}

impl<T> Default for SplitAllocatedVec<T> {
    fn default() -> Self {
        Self { 
            allocations: vec![],
            minimum_allocation_size: 32,
            full_allocations: vec![],
        }
    } 
}
