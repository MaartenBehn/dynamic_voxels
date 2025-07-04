
use core::fmt;
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

impl<T: Copy + Default + fmt::Debug> AllocatedVec<T> {
    pub fn new(minimum_allocation_size: usize) -> Self {
        Self { 
            allocations: vec![],
            minimum_allocation_size,
        }
    } 

    pub fn push(&mut self, mut values: &[T], allocator: &mut BuddyBufferAllocator) -> OctaResult<usize> {

        // Fill exsiting allocations
        for alloc in self.allocations.iter_mut() {
            for (start, end) in alloc.free_ranges.iter_mut() {
                let len = *end - *start;
                if values.len() > len {
                    continue;
                }

                let new_start = *start + values.len();
                alloc.data[*start..new_start].copy_from_slice(values);

                alloc.changed_ranges.push((*start, new_start));
                let res = alloc.start_index + *start; 

                (*start) = new_start;
                alloc.needs_free_optimization = true;

                return Ok(res);
            }
        }

        let allocation = allocator.alloc(self.minimum_allocation_size.max(values.len() * size_of::<T>()))?;

        let capacity = allocation.size / size_of::<T>();

        let mut padding = allocation.start % size_of::<T>();
        let mut start_index = allocation.start / size_of::<T>();;
        if padding != 0 {
            padding = size_of::<T>() - padding;
            start_index += 1;
        }

        let mut data = Vec::with_capacity(capacity); 
        data.extend_from_slice(values);
        data.resize(capacity, T::default());

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

    pub fn flush(&mut self, buffer: &mut Buffer) {
        for alloc in self.allocations.iter_mut() {
            alloc.optimize_changed_ranges();

            for (start, end) in alloc.changed_ranges.iter() {
                buffer.copy_data_to_buffer_without_aligment(&alloc.data[*start..*end], alloc.allocation.start + start + alloc.padding);
            }
            alloc.changed_ranges.clear();
        }
    }

    pub fn optimize(&mut self) {
        for alloc in self.allocations.iter_mut() {
            alloc.optimize_free_ranges();            
        }
    }

    pub fn get_memory_size(&self) -> usize {
        self.allocations.iter()
            .map(|alloc| alloc.allocation.size )
            .sum()
    }

    pub fn get_num_allocations(&self) -> usize {
        self.allocations.len()
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
            } else if last_range.0 == last_range.1 {
                self.free_ranges.swap_remove(i+1);
            }
        }

        if !self.free_ranges.is_empty() && self.free_ranges[0].0 == self.free_ranges[0].1 {
            self.free_ranges.swap_remove(0);
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
