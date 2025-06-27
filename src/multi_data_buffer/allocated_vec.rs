
use std::{iter};

use octa_force::{log::error, vulkan::Buffer, OctaResult};

use super::{buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}};


#[derive(Debug)]
pub struct AllocatedVec<T> {
    allocations: Vec<Allocation<T>>,
    pub minimum_allocation_size: usize,
}

#[derive(Debug)]
struct Allocation<T> {
    allocation: BuddyAllocation,
    start_index: usize,
    padding: usize,
    data: Vec<T>,
    free_ranges: Vec<(usize, usize)>,
    changed_ranges: Vec<(usize, usize)>,
    needs_free_optimization: bool,
}

impl<T: Copy + Default> AllocatedVec<T> {
    pub fn push(&mut self, mut values: &[T], allocator: &mut BuddyBufferAllocator) -> OctaResult<usize> {

        // Fill exsiting allocations
        for alloc in self.allocations.iter_mut() {
            while let Some((start, end)) = alloc.free_ranges.pop() {
                let len = end - start;
                if values.len() >= len {
                    continue;
                }

                let new_start = start + values.len();
                alloc.data[start..new_start].copy_from_slice(values);
                alloc.free_ranges.push((new_start, end));
                alloc.changed_ranges.push((start, new_start));

                return Ok(alloc.start_index + start);
            }
        }

        let allocation = allocator.alloc(self.minimum_allocation_size.max(values.len()))?;

        let capacity = allocation.size / size_of::<T>();
        let padding = allocation.start % size_of::<T>();
        let start_index = allocation.start / size_of::<T>() + (padding != 0) as usize; 
        let mut data = Vec::with_capacity(capacity); 
        data.extend_from_slice(values);
        data[(values.len()..)].fill(T::default());

        if values.len() < capacity {
            self.allocations.push(Allocation {
                allocation,
                start_index,
                padding,
                data,
                free_ranges: vec![(values.len(), capacity)], 
                changed_ranges: vec![(0, values.len())],
                needs_free_optimization: false,
            });
        } else {
            self.allocations.push(Allocation {
                allocation,
                start_index,
                padding,
                data,
                free_ranges: vec![],
                changed_ranges: vec![(0, capacity)],
                needs_free_optimization: false,
            });
        }

        Ok(start_index)
    }

    pub fn remove(&mut self, index: usize, size: usize) {
        let res = self.allocations.iter_mut()
            .find(|a| a.start_index <= index && (a.start_index + a.data.len()) > index + size);

        if res.is_some() {
            let alloc = res.unwrap();
            let index = index - alloc.start_index;
            alloc.free_ranges.push((index, index + size));
            alloc.needs_free_optimization = true;
            return;
        }  

        error!("Allocated Index {index} not found!");
    }

    pub fn flush(&mut self, buffer: &mut Buffer) -> OctaResult<()> {
        for alloc in self.allocations.iter_mut() {
            alloc.optimize_changed_ranges();

            for (start, end) in alloc.changed_ranges.iter() {
                buffer.copy_data_to_buffer_without_aligment(&alloc.data[*start..*end], alloc.allocation.start + start + alloc.padding);
            }
        }

        Ok(())
    }

    pub fn optimize(&mut self) {
        for alloc in self.allocations.iter_mut() {
            alloc.optimize_free_ranges();            
        }
    }
}

impl<T> Allocation<T> {
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

impl<T> Default for AllocatedVec<T> {
    fn default() -> Self {
        Self { 
            allocations: vec![],
            minimum_allocation_size: 32,
        }
    } 
}
