
use core::fmt;
use std::{iter, marker::PhantomData};
use rayon::{iter::empty, prelude::*};

use octa_force::{anyhow::bail, itertools::Itertools, log::{debug, error}, vulkan::Buffer, OctaResult};


#[derive(Debug)]
pub struct CachedVec<T, Hasher = fnv::FnvBuildHasher> {
    data: Vec<T>,
    used_ranges: Vec<(usize, usize)>,
    cache: hashbrown::HashTable<CompactRange>,
    _phantom: std::marker::PhantomData<Hasher>,
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

impl<T: Copy + Default + fmt::Debug + Sync + Eq + std::hash::Hash, Hasher: std::hash::BuildHasher + Default + fmt::Debug> 
    CachedVec<T, Hasher> {
    pub fn new(size: usize) -> Self {
        Self { 
            data: vec![T::default(); size],
            used_ranges: vec![],
            cache: Default::default(),
            _phantom: Default::default()
        }
    } 

    pub fn push(&mut self, mut values: &[T]) -> OctaResult<u32> {
        if values.is_empty() {
            return Ok(0);
        }
        
        let hasher = Hasher::default();
        let hash = hasher.hash_one(values);

        if self.used_ranges.is_empty() {
            let end = values.len();

            self.data[0..end].copy_from_slice(&values);
            self.used_ranges.push((0, end));

            let range =  CompactRange {
                start: 0 as u32,
                length: values.len() as u8,
            };
            self.cache.insert_unique(hash, range, |r| hasher.hash_one(&self.data[r.as_range()]));

            return Ok(0);
        }

        let res = self.cache.find(hash,
            |compact_range| &self.data[compact_range.as_range()] == values);

        if let Some(r) = res {
            return Ok(r.start);
        }


        let mut smallest_free_range = None;
        let (last_start, last_end) = self.used_ranges.last().unwrap();

        // Find the best used range where a prefix fits
        let res = self.used_ranges.iter()
            .tuple_windows::<(_, _)>()
            .enumerate()
            .map(|(i, ((a_start, a_end), (b_start, _)))| (i, *a_start, *a_end, *b_start - *a_end))
            // Add the last used range with the space to the end
            .chain(iter::once((self.used_ranges.len() -1, *last_start, *last_end, self.data.len() - *last_end)))
            .map(|(used_range_index, start, end, free_range_size)| {

                if free_range_size >= values.len() {
                    if let Some((_, _, free_size)) = smallest_free_range {
                        if free_size > free_range_size {
                            smallest_free_range = Some((end, used_range_index, free_range_size));
                        }
                    } else {
                        smallest_free_range = Some((end, used_range_index, free_range_size));
                    }
                } 

                let min = values.len().saturating_sub(free_range_size).max(1);
                for hits in (min..=values.len()).rev() {
                    let slice_to_match = &values[..hits];

                    if self.data[start..end].ends_with(slice_to_match) {
                        return (hits, used_range_index);
                    }
                }
                (0, 0)
            })
            .max_by(|a, b| a.0.cmp(&b.0));

        if let Some((hits, used_range_index)) = res {
            let (range_start, range_end) = &mut self.used_ranges[used_range_index];
            let start = *range_end - hits;
            let end = start + values.len();

            self.data[*range_end..end].copy_from_slice(&values[hits..]);
            (*range_end) = end;

            let range =  CompactRange {
                start: start as u32,
                length: values.len() as u8,
            };
            self.cache.insert_unique(hash, range, |r| hasher.hash_one(&self.data[r.as_range()]));

            let next_used_range = self.used_ranges.get_mut(used_range_index + 1);
            if let Some((next_start, _)) = next_used_range {
                if end >= *next_start {
                    (*next_start) = start;
                    self.used_ranges.remove(used_range_index);
                }
            }

            return Ok(start as u32);
        }

        if let Some((start, used_range_index,_)) = smallest_free_range {
            let end = start + values.len();
            let (range_start, range_end) = &mut self.used_ranges[used_range_index];

            self.data[start..end].copy_from_slice(&values);
            (*range_end) = end;

            let range =  CompactRange {
                start: start as u32,
                length: values.len() as u8,
            };
            self.cache.insert_unique(hash, range, |r| hasher.hash_one(&self.data[r.as_range()]));

            let next_used_range = self.used_ranges.get_mut(used_range_index + 1);
            if let Some((next_start, _)) = next_used_range {
                if end >= *next_start {
                    (*next_start) = start;
                    self.used_ranges.remove(used_range_index);
                }
            }

            return Ok(start as u32);
        }

        bail!("Could not find free enought space in cached vector!");
    }

    pub fn get(&self, index: u32) -> T {
        self.data[index as usize]
    }

    pub fn get_range(&self, r: std::ops::Range<usize>) -> &[T] {
        &self.data[r]
    }

    pub fn set(&mut self, index: usize, data: &[T]) {
        let max = index + data.len();
        self.data[index..max].copy_from_slice(data);
    }

    pub fn flush(&mut self, buffer: &mut Buffer) {
        buffer.copy_data_to_buffer_without_aligment(&self.data, 0);
    }

    pub fn get_memory_size(&self) -> usize {
        self.data.len() * size_of::<T>()
    }
}

impl<T: Eq> PartialEq for CachedVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data 
        && self.used_ranges == other.used_ranges 
    }
}
