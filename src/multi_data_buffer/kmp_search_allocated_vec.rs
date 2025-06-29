
use core::fmt;
use std::{iter};
use rayon::prelude::*;

use octa_force::{itertools::Itertools, log::{debug, error}, vulkan::Buffer, OctaResult};

use super::{buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, kmp::{kmp_find, kmp_find_prefix_with_lsp_table, kmp_find_with_lsp_table, kmp_table}};


#[derive(Debug)]
pub struct KmpSearchAllocatedVec<T> {
    allocations: Vec<KmpSearchAllocation<T>>,
    pub minimum_allocation_size: usize,
}

#[derive(Debug)]
struct KmpSearchAllocation<T> {
    allocation: BuddyAllocation,
    start_index: usize,
    padding: usize,
    data: Vec<T>,
    used_ranges: Vec<(usize, usize)>,
    needs_free_optimization: bool,
}

impl<T: Copy + Default + fmt::Debug + Sync + Eq> KmpSearchAllocatedVec<T> {
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

        let kmp_lsp = kmp_table(values);

        let res = self.allocations.iter_mut()
            .find_map(|alloc| {
                alloc.used_ranges.iter()
                    .find_map(|(start, end)| {
                        kmp_find_with_lsp_table(values, &alloc.data[*start..*end], &kmp_lsp)
                    })
                    .map(|start| (alloc, start))
            });

        if let Some((alloc, start)) = res {
            return Ok(alloc.start_index + start);
        }

        // TODO find with best hits
        let res = self.allocations.iter_mut()
            .find_map(|alloc| {

                let (last_start, last_end) = alloc.used_ranges.last().unwrap();

                alloc.used_ranges.iter()
                    .tuple_windows::<(_, _)>()
                    .enumerate()
                    .map(|(i, ((a_start, a_end), (b_start, _)))| (i, *a_start, *a_end, *b_start - *a_end))
                    .chain(iter::once((alloc.used_ranges.len() -1, *last_start, *last_end, alloc.data.len() - *last_end)))
                    .find_map(|(i, start, end, size)| {
                        kmp_find_prefix_with_lsp_table(
                            values, 
                            &alloc.data[start..end], 
                            size, &kmp_lsp)
                        .map(|(start, hits)| (start, i))
                    })
                    .map(|(start, i)| (alloc, start, i))
            });

        if let Some((alloc, start, used_range_index)) = res {
            let end = start + values.len();
            let (range_start, range_end) = &mut alloc.used_ranges[used_range_index];

            let wild_card_size = *range_end - start;
            alloc.data[*range_end..end].copy_from_slice(&values[wild_card_size..]);
            (*range_end) = end;

            let next_used_range =  alloc.used_ranges.get_mut(used_range_index + 1);
            if let Some((next_start, _)) = next_used_range {
                if end >= *next_start {
                    (*next_start) = start;
                    alloc.used_ranges.remove(used_range_index);
                }
            }

            return Ok(alloc.start_index + start);
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
            self.allocations.push(KmpSearchAllocation {
                allocation,
                start_index,
                padding,
                data,
                used_ranges: vec![(0, values.len())], 
                needs_free_optimization: false,
            });
        } else {
            self.allocations.push(KmpSearchAllocation {
                allocation,
                start_index,
                padding,
                data,
                used_ranges: vec![(0, capacity)],
                needs_free_optimization: false,
            });
        }

        Ok(start_index)
    }

    pub fn remove(&mut self, index: usize, size: usize) {
        /*
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
*/
    }

    pub fn flush(&mut self, buffer: &mut Buffer) {
        for alloc in self.allocations.iter_mut() {
            buffer.copy_data_to_buffer_without_aligment(&alloc.data, alloc.allocation.start + alloc.padding);
        }
    }

    pub fn optimize(&mut self) {
    
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
