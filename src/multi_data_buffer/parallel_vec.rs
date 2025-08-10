
use core::fmt;
use std::{iter, marker::PhantomData, sync::Arc};
use parking_lot::RwLock;
use rayon::{iter::empty, prelude::*};

use octa_force::{anyhow::bail, itertools::Itertools, log::{debug, error}, vulkan::Buffer, OctaResult};

use crate::multi_data_buffer::cached_vec::CompactRange;

use super::cached_vec::CachedVec;

#[derive(Debug, Clone)]
pub struct ParallelVec<T, Hasher = fnv::FnvBuildHasher> {
    inner: Arc<RwLock<CachedVec<T, Hasher>>>
}

impl<T: Copy + Default + fmt::Debug + Eq + std::hash::Hash> CachedVec<T> {
    pub fn parallel(self) -> ParallelVec<T>{
        ParallelVec { inner: Arc::new(RwLock::new(self)) }
    }
}

impl<T: Copy + Default + fmt::Debug + Eq + std::hash::Hash, Hasher: std::hash::BuildHasher + Default + fmt::Debug> 
    ParallelVec<T, Hasher> {

    pub fn single(self) -> CachedVec<T, Hasher> {
        Arc::try_unwrap(self.inner).unwrap().into_inner()
    }
    
    pub fn push(&self, mut values: &[T]) -> OctaResult<u32> {
        if values.is_empty() {
            return Ok(0);
        }
 
        let hasher = Hasher::default();
        let hash = hasher.hash_one(values);

        let mut inner_r = self.inner.upgradable_read();
        if inner_r.used_ranges.is_empty() {
            inner_r.with_upgraded(|inner_w| {
                let end = values.len();

                // Write
                inner_w.data[0..end].copy_from_slice(&values);
                inner_w.used_ranges.push((0, end));

                let range = CompactRange {
                    start: 0 as u32,
                    length: values.len() as u8,
                };
                inner_w.cache.insert_unique(hash, range, |r| hasher.hash_one(&inner_w.data[r.as_range()]));
            });

            return Ok(0);
        }

        let res = inner_r.cache.find(hash,
            |compact_range| &inner_r.data[compact_range.as_range()] == values);

        if let Some(r) = res {
            return Ok(r.start);
        }

        let mut smallest_free_range = None;
        let (last_start, last_end) = inner_r.used_ranges.last().unwrap();

        // Find the best used range where a prefix fits
        let res = inner_r.used_ranges.iter()
            .tuple_windows::<(_, _)>()
            .enumerate()
            .map(|(i, ((a_start, a_end), (b_start, _)))| (i, *a_start, *a_end, *b_start - *a_end))
            // Add the last used range with the space to the end
            .chain(iter::once((inner_r.used_ranges.len() -1, *last_start, *last_end, inner_r.data.len() - *last_end)))
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

                    if inner_r.data[start..end].ends_with(slice_to_match) {
                        return (hits, used_range_index);
                    }
                }
                (0, 0)
            })
            .max_by(|a, b| a.0.cmp(&b.0));

        if let Some((hits, used_range_index)) = res {
            return Ok(inner_r.with_upgraded(|inner_w| {
                let (range_start, range_end) = &mut inner_w.used_ranges[used_range_index];
                let start = *range_end - hits;
                let end = start + values.len();

                inner_w.data[*range_end..end].copy_from_slice(&values[hits..]);
                (*range_end) = end;

                let range =  CompactRange {
                    start: start as u32,
                    length: values.len() as u8,
                };
                inner_w.cache.insert_unique(hash, range, |r| hasher.hash_one(&inner_w.data[r.as_range()]));

                let next_used_range = inner_w.used_ranges.get_mut(used_range_index + 1);
                if let Some((next_start, _)) = next_used_range {
                    if end >= *next_start {
                        (*next_start) = start;
                        inner_w.used_ranges.remove(used_range_index);
                    }
                }

                start as u32
            }))
        }

        if let Some((start, used_range_index,_)) = smallest_free_range {
            return Ok(inner_r.with_upgraded(|inner_w| {
                let end = start + values.len();
                let (range_start, range_end) = &mut inner_w.used_ranges[used_range_index];

                inner_w.data[start..end].copy_from_slice(&values);
                (*range_end) = end;

                let range =  CompactRange {
                    start: start as u32,
                    length: values.len() as u8,
                };
                inner_w.cache.insert_unique(hash, range, |r| hasher.hash_one(&inner_w.data[r.as_range()]));

                let next_used_range = inner_w.used_ranges.get_mut(used_range_index + 1);
                if let Some((next_start, _)) = next_used_range {
                    if end >= *next_start {
                        (*next_start) = start;
                        inner_w.used_ranges.remove(used_range_index);
                    }
                }

                start as u32
            }));
        }

        bail!("Could not find free enought space in cached vector!");
    }

    pub fn get(&self, index: u32) -> T {
        let inner_r = self.inner.read();
        inner_r.data[index as usize]
    }

    pub fn get_range(&self, r: std::ops::Range<usize>) -> Vec<T> {
        let inner_r = self.inner.read();
        inner_r.data[r].to_vec()
    }

    pub fn set(&mut self, index: usize, data: &[T]) {
        let max = index + data.len();
        
        let mut inner_w = self.inner.write();
        inner_w.data[index..max].copy_from_slice(data);
    }

    pub fn flush(&mut self, buffer: &mut Buffer) {
        let inner_r = self.inner.read();
        buffer.copy_data_to_buffer_without_aligment(&inner_r.data, 0);
    }

    pub fn get_memory_size(&self) -> usize {
        let inner_r = self.inner.read();
        inner_r.data.len() * size_of::<T>()
    }
}
