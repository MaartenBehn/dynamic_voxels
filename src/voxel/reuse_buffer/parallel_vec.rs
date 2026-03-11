
use core::fmt;
use std::{cell::UnsafeCell, hash::{BuildHasher, Hash}, iter, marker::PhantomData, sync::{Arc, atomic::{AtomicUsize, Ordering}}};
use dashmap::{DashMap, Entry};
use fnv::FnvHasher;
use parking_lot::RwLock;
use rayon::{iter::empty, prelude::*};

use octa_force::{anyhow::bail, itertools::Itertools, log::{debug, error}, vulkan::Buffer, OctaResult};
use smallvec::{SmallVec, ToSmallVec, smallvec};

use crate::{model::template::value, scene::staging_copies::SceneStagingBuilder, voxel::reuse_buffer::cached_vec::CompactRange};

use super::cached_vec::CachedVec;

#[derive(Debug, Clone)]
pub struct ParallelVec<T> {
    inner: Arc<InnerParallelVec<T>>,
}

#[derive(Debug)]
struct InnerParallelVec<T> {
    data: UnsafeCell<Vec<T>>,
    write_head: AtomicUsize,
    flushed: AtomicUsize,
    cache: DashMap<u64, SmallVec<[CompactRange; 2]>, nohash_hasher::BuildNoHashHasher<u64>>,
}

impl<T: Copy + Default + fmt::Debug + Eq + std::hash::Hash> ParallelVec<T> {
    pub fn new(size: usize) -> ParallelVec<T>{
        ParallelVec { inner: Arc::new(InnerParallelVec { 
            data: UnsafeCell::new(vec![T::default(); size]), 
            write_head: AtomicUsize::new(0),
            flushed: AtomicUsize::new(0),
            cache: DashMap::with_hasher(nohash_hasher::BuildNoHashHasher::new()),
        }) }
    }

    fn get_data(&self) -> &mut [T] {
        unsafe { &mut *self.inner.data.get() }
    }

    pub fn push(&self, values: &[T]) -> OctaResult<u32> {
        if values.is_empty() {
            return Ok(0);
        }

        let mut hasher = fnv::FnvBuildHasher::default();
        let hash = hasher.hash_one(values);
        
        let data = self.get_data();

        match self.inner.cache.entry(hash) {
            Entry::Occupied(mut e) => {
                let vec = e.get_mut();

                if let Some(r) = vec.iter().find(|r| &data[r.as_range()] == values) {
                    return Ok(r.start);
                }
                
                self.insert_data(values, vec, hash)
            }
            Entry::Vacant(e) => {
                let mut e = e.insert(SmallVec::new()); 
                let vec = e.value_mut();
                
                self.insert_data(values, vec, hash)
            }
        }
    }

    fn insert_data(&self, values: &[T], vec: &mut SmallVec<[CompactRange; 2]>, hash: u64) -> OctaResult<u32> {
        
        let start = self.inner.write_head.fetch_add(values.len(), Ordering::Relaxed);
        let end = start + values.len();

        let data = unsafe { &mut *self.inner.data.get() };
        data[start..end].copy_from_slice(values);

        vec.push(CompactRange {
            start: start as _,
            length: values.len() as _,
        });

        return Ok(start as _);
    }
    
    pub fn get(&self, index: u32) -> T {
        self.get_data()[index as usize]
    }

    pub fn get_range(&self, r: std::ops::Range<usize>) -> &[T] {
        &self.get_data()[r]
    }

    pub fn push_scene_builder(&self, builder: &mut SceneStagingBuilder, offset: usize) {
        
        let flushed = self.inner.flushed.load(Ordering::Relaxed);
        let head = self.inner.write_head.load(Ordering::Relaxed);
        self.inner.flushed.store(head, Ordering::Relaxed);

        builder.push(&self.get_data()[flushed..head], offset + flushed * size_of::<T>());
    }
 
    pub fn data(&self) -> &[T] {
        self.get_data()
    }

    pub fn get_memory_size(&self) -> usize {
        self.get_data().len() * size_of::<T>()
    }

    pub fn filled(&self) -> f32 {
        let head = self.inner.write_head.load(Ordering::Relaxed) as f32;
        head / self.get_data().len() as f32
    }

    pub fn reset(&mut self) { 
        self.inner.write_head.store(0, Ordering::Relaxed);
        self.inner.flushed.store(0, Ordering::Relaxed);
        self.inner.cache.clear();
    }
}

unsafe impl<T: Send> Sync for InnerParallelVec<T> {}
