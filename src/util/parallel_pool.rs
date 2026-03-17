use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;

#[derive(Debug, Default)]
pub struct ParallelPool<T: Default> {
    slots: Vec<UnsafeCell<T>>,
    next: Vec<AtomicUsize>,
    free_head: AtomicUsize,
}

unsafe impl<T: Default + Send> Send for ParallelPool<T> {}
unsafe impl<T: Default + Send> Sync for ParallelPool<T> {}

impl<T: Default> ParallelPool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut next = Vec::with_capacity(capacity);

        for i in 0..capacity {
            next.push(AtomicUsize::new(i + 1));
        }

        Self {
            slots: (0..capacity).map(|_| UnsafeCell::new(T::default())).collect(),
            next,
            free_head: AtomicUsize::new(0),
        }
    }

    pub fn insert(&self, value: T) -> Option<usize> {
        loop {
            let head = self.free_head.load(Ordering::Acquire);

            if head >= self.slots.len() {
                return None;
            }

            let next = self.next[head].load(Ordering::Relaxed);

            if self.free_head
                .compare_exchange(head, next, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                unsafe {
                    *self.slots[head].get() = value;
                }

                return Some(head);
            }
        }
    }

    pub fn remove(&self, index: usize) {
        unsafe {
            *self.slots[index].get() = T::default();
        }

        loop {
            let head = self.free_head.load(Ordering::Acquire);

            self.next[index].store(head, Ordering::Relaxed);

            if self.free_head
                .compare_exchange(head, index, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
        }
    }

    pub fn get(&self, index: usize) -> &T {
        unsafe { &*self.slots[index].get() }
    }

    pub fn get_mut(&self, index: usize) -> &mut T {
        unsafe { &mut *self.slots[index].get() }
    }
}
