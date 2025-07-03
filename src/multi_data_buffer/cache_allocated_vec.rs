
use core::fmt;
use std::{iter, marker::PhantomData};
use rayon::{iter::empty, prelude::*};

use octa_force::{itertools::Itertools, log::{debug, error}, vulkan::Buffer, OctaResult};

use super::{buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, kmp::{kmp_find, kmp_find_prefix_with_lsp_table, kmp_find_with_lsp_table, kmp_table}};

const EXTRA_FULL_CHECK: bool = false;
const KMP_PREFIX: bool = false;

#[derive(Debug)]
pub struct CacheAllocatedVec<T, Hasher = fnv::FnvBuildHasher> {
    allocations: Vec<CacheAllocation<T>>,
    pub minimum_allocation_size: usize,
    _phantom: std::marker::PhantomData<Hasher>,
}

#[derive(Debug)]
struct CacheAllocation<T> {
    allocation: BuddyAllocation,
    start_index: usize,
    padding: usize,
    data: Vec<T>,
    used_ranges: Vec<(usize, usize)>,
    cache: hashbrown::HashTable<CompactRange>,
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

impl<T: Copy + Default + fmt::Debug + Sync + Eq + std::hash::Hash, Hasher: std::hash::BuildHasher + Default + fmt::Debug> CacheAllocatedVec<T, Hasher> {
    pub fn new(minimum_allocation_size: usize) -> Self {
        Self { 
            allocations: vec![],
            minimum_allocation_size,
            _phantom: Default::default()
        }
    } 

    pub fn push(&mut self, mut values: &[T], allocator: &mut BuddyBufferAllocator) -> OctaResult<usize> {
        if values.is_empty() {
            return Ok(0);
        }

        let hasher = Hasher::default();
        let hash = hasher.hash_one(values);
        let res = self.allocations.iter()
            .find_map(|alloc| {
                alloc.cache
                    .find(
                        hash,
                        |compact_range| &alloc.data[compact_range.as_range()] == values,
                    )
                    .map(|r| (r, alloc))
            });

        if let Some((r, alloc)) = res {
            return Ok(alloc.start_index + r.start as usize);
        }

        let mut smallest_free_range = None;

        // Find the best used range where a prefix fits
        let res = self.allocations.iter_mut()
            .enumerate()
            .map(|(alloc_index, alloc)| {
                let (last_start, last_end) = alloc.used_ranges.last().unwrap();

                alloc.used_ranges.iter()
                    .tuple_windows::<(_, _)>()
                    .enumerate()
                    .map(|(i, ((a_start, a_end), (b_start, _)))| (i, *a_start, *a_end, *b_start - *a_end))
                    // Add the last used range with the space to the end
                    .chain(iter::once((alloc.used_ranges.len() -1, *last_start, *last_end, alloc.data.len() - *last_end)))
                    .map(|(used_range_index, start, end, free_range_size)| {
                        
                        if free_range_size >= values.len() {
                            if let Some((_ , _, _, free_size)) = smallest_free_range {
                                if free_size > free_range_size {
                                    smallest_free_range = Some((alloc_index, end, used_range_index, free_range_size));
                                }
                            } else {
                                smallest_free_range = Some((alloc_index, end, used_range_index, free_range_size));
                            }
                        } 

                        let min = values.len().saturating_sub(free_range_size).max(1);
                        for hits in (min..=values.len()).rev() {
                            let slice_to_match = &values[..hits];

                            if alloc.data[start..end].ends_with(slice_to_match) {
                                return (hits, used_range_index);
                            }
                        }
                        (0, 0)
                    })
                    .max_by(|a, b| a.0.cmp(&b.0))
                    .map(|(hits, used_range_index)| (hits, Some(alloc), used_range_index))
                    .unwrap_or((0, None, 0))
            })
            .max_by(|a, b| a.0.cmp(&b.0))
            .map(|v| if v.0 != 0 { Some(v) } else { None })
            .flatten();

        if let Some((hits, alloc, used_range_index)) = res {
            let alloc = alloc.unwrap();
            let (range_start, range_end) = &mut alloc.used_ranges[used_range_index];
            let start = *range_end - hits;
            let end = start + values.len();

            alloc.data[*range_end..end].copy_from_slice(&values[hits..]);
            (*range_end) = end;

            let range =  CompactRange {
                start: start as u32,
                length: values.len() as u8,
            };
            alloc.cache.insert_unique(hash, range, |r| hasher.hash_one(&alloc.data[r.as_range()]));

            let next_used_range =  alloc.used_ranges.get_mut(used_range_index + 1);
            if let Some((next_start, _)) = next_used_range {
                if end >= *next_start {
                    (*next_start) = start;
                    alloc.used_ranges.remove(used_range_index);
                }
            }

            return Ok(alloc.start_index + start);
        }

        if let Some((alloc_nr, start, used_range_index,_)) = smallest_free_range {
            let end = start + values.len();
            let alloc = &mut self.allocations[alloc_nr];
            let (range_start, range_end) = &mut alloc.used_ranges[used_range_index];

            alloc.data[start..end].copy_from_slice(&values);
            (*range_end) = end;

            let range =  CompactRange {
                start: start as u32,
                length: values.len() as u8,
            };
            alloc.cache.insert_unique(hash, range, |r| hasher.hash_one(&alloc.data[r.as_range()]));

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

        let mut cache: hashbrown::HashTable<CompactRange> = Default::default();
        let range =  CompactRange {
            start: 0,
            length: values.len() as u8,
        };
        cache.insert_unique(hash, range, |r| hasher.hash_one(&data[r.as_range()]));


        if values.len() < capacity {
            self.allocations.push(CacheAllocation {
                allocation,
                start_index,
                padding,
                data,
                used_ranges: vec![(0, values.len())],
                cache
            });
        } else {
            self.allocations.push(CacheAllocation {
                allocation,
                start_index,
                padding,
                data,
                used_ranges: vec![(0, capacity)],
                cache
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
