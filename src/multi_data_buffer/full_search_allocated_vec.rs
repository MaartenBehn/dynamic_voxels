
use core::fmt;
use std::{iter};
use rayon::prelude::*;

use octa_force::{log::{debug, error}, vulkan::Buffer, OctaResult};

use super::{buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}};


#[derive(Debug)]
pub struct FullSearchAllocatedVec<T> {
    allocations: Vec<FullSearchAllocation<T>>,
    pub minimum_allocation_size: usize,
}

#[derive(Debug)]
struct FullSearchAllocation<T> {
    allocation: BuddyAllocation,
    start_index: usize,
    padding: usize,
    data: Vec<T>,
    free_ranges: Vec<(usize, usize)>,
    needs_free_optimization: bool,
}

impl<T: Copy + Default + fmt::Debug + Sync + Eq> FullSearchAllocatedVec<T> {
    pub fn new(minimum_allocation_size: usize) -> Self {
        Self { 
            allocations: vec![],
            minimum_allocation_size,
        }
    } 

    pub fn push(&mut self, mut values: &[T], allocator: &mut BuddyBufferAllocator) -> OctaResult<usize> {
        if values.is_empty() {
            return Ok(0);
        }

        let res = self.allocations.par_iter()
            .enumerate()
            .filter(|(_, alloc)|  alloc.data.len() >= values.len())
            .map(|(alloc_nr, alloc)| {
                alloc.data.par_windows(values.len())
                    .enumerate()
                    .map(|(window_start, w)| {
                        let window_end = window_start + values.len();

                        let touching_free_ranges = alloc.free_ranges.par_iter()
                            .filter(|(start, end)| *start < window_end && *end > window_start)
                            .collect::<Vec<_>>();

                        let hits = if touching_free_ranges.is_empty() && w == values {
                            Some(values.len())
                        } else if touching_free_ranges.len() == 1 
                            && touching_free_ranges[0].0 <= window_start 
                            && touching_free_ranges[0].1 >= window_end {
                            if touching_free_ranges[0].0 == window_start {
                                Some(0)
                            } else {
                                None
                            }
                        } else {
                            w.par_iter()
                                .zip(values)
                                .enumerate()
                                .map(|(i, (w, v))| {
                                    if (touching_free_ranges.par_iter()
                                        .find_any(|(start, end)| i >= *start && i < *end))
                                        .is_some() {

                                        Some(0)
                                    } else if w == v {
                                        Some(1)
                                    } else {
                                        None
                                    }
                                })
                                .reduce(||Some(0), |a, b|  {
                                    if a.is_none() || b.is_none() {
                                        None
                                    } else if a.is_none() {
                                        b
                                    } else if b.is_none() {
                                        a
                                    } else {
                                        Some(a.unwrap() + b.unwrap())
                                    }
                                })
                        };

                        (alloc_nr, window_start, hits)
                    })
                    .max_by(|(_, _, a_hits), (_, _, b_hits)| {
                        a_hits.cmp(b_hits)
                    })
                    .unwrap_or((0, 0, None))
            })
            .max_by(|(_, _, a_hits), (_, _, b_hits)| {
                a_hits.cmp(b_hits)
            })
            .unwrap_or((0, 0, None));
       
        if let (alloc_nr, window_start, Some(hits)) = res {
            let window_end = window_start + values.len();

            let alloc = &mut self.allocations[alloc_nr];
            alloc.data[window_start..window_end].copy_from_slice(values);

            alloc.free_ranges.par_iter_mut()
                .for_each(|(start, end)| {
                    if window_start <= *start && *end <= window_end {
                        (*end) = (*start);
                    } else if window_start <= *start && window_end <= *end {
                        (*start) = window_end;
                    } else if *start <= window_start && *end <= window_end {
                        (*end) = window_start;
                    }
                });

            alloc.needs_free_optimization = true; 
            return Ok(alloc.start_index + window_start);
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
            self.allocations.push(FullSearchAllocation {
                allocation,
                start_index,
                padding,
                data,
                free_ranges: vec![(values.len(), capacity)], 
                needs_free_optimization: false,
            });
        } else {
            self.allocations.push(FullSearchAllocation {
                allocation,
                start_index,
                padding,
                data,
                free_ranges: vec![],
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
            buffer.copy_data_to_buffer_without_aligment(&alloc.data, alloc.allocation.start + alloc.padding);
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

impl<T> FullSearchAllocation<T> {
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
}
