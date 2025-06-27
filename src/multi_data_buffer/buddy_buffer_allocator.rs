use octa_force::anyhow::bail;
use octa_force::egui::mutex::Mutex;
use octa_force::log::{debug, error, trace};
use octa_force::OctaResult;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::sync::Arc;

#[derive(Clone)]
pub struct BuddyBufferAllocator {
    inner: Arc<Mutex<BuddyAllocator>>,
}

#[derive(Clone)]
pub struct BuddyAllocation {
    inner: Arc<Mutex<BuddyAllocator>>,
    pub start: usize,
    pub size: usize,
} 

#[derive(Debug)]
struct BuddyAllocator {
    free_list: Vec<Vec<(usize, usize)>>,
    mp: HashMap<usize, usize>,
    pub size: usize,
    min_n: usize,
}

impl BuddyBufferAllocator {
    pub fn new(size: usize, min_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BuddyAllocator::new(size, min_size))),
        }
    }

    pub fn alloc(&mut self, size: usize) -> OctaResult<BuddyAllocation> {
        let (start, size) = self.inner.lock().alloc(size)?;

        Ok(BuddyAllocation {
            inner: self.inner.to_owned(),
            start,
            size,
        })
    }
}

impl Drop for BuddyAllocation {
    fn drop(&mut self) {
        let res = self.inner.lock().dealloc(self.start);
        if let Err(e) = res {
            error!("Buddy Allocation Drop failed with: {e}")
        }
    }
}

impl BuddyAllocator {
    fn new(size: usize, min_size: usize) -> Self {
        let n = calc_n(size);        
        let min_n = calc_n(min_size);

        let free_list: Vec<_> = iter::repeat(vec![])
            .take(n - min_n)
            .chain([vec![(0, size - 1)]].into_iter())
            .collect();

        BuddyAllocator {
            free_list,
            mp: Default::default(),
            size,
            min_n,
        }
    }

    fn clear(&mut self) {
        self.free_list.iter_mut().for_each(|l| l.clear()); 
        self.free_list.last_mut().unwrap().push((0, self.size - 1));
        self.mp.clear();
    }

    // From https://www.geeksforgeeks.org/buddy-memory-allocation-program-set-1-allocation/
    /// In: size in byte
    /// Out: start index and size of allocation
    fn alloc(&mut self, size: usize) -> OctaResult<(usize, usize)> {
        // Calculate index in free list
        // to search for block if available
        let n = calc_n(size).max(self.min_n) - self.min_n;

        let space = if !self.free_list[n].is_empty() {
            self.free_list[n].remove(0)
        } else {
            let found = self.free_list[(n + 1)..]
                .iter_mut()
                .enumerate()
                .find_map(|(i, free)| {
                    if !free.is_empty() {
                        Some((i + n + 1, free.remove(0)))
                    } else {
                        None
                    }
                });

            if found.is_none() {
                bail!("No free Space found")
            }
            let (found_n, mut temp) = found.unwrap();

            for i in (n..found_n).rev() {
                // Divide block into two halves
                let pair1 = (temp.0, temp.0 + (temp.1 - temp.0) / 2);
                let pair2 = (temp.0 + (temp.1 - temp.0 + 1) / 2, temp.1);

                self.free_list[i].push(pair1);
                self.free_list[i].push(pair2);
                temp = self.free_list[i].remove(0);
            }

            temp
        };

        // map starting address with
        // size to make deallocating easy
        debug!("Memory from {} to {} for {} bytes allocated", space.0, space.1, size);

        let size = space.1 - space.0 + 1;
        self.mp.insert(space.0, size);

        Ok((space.0, size))
    }

    // From https://www.geeksforgeeks.org/buddy-memory-allocation-program-set-2-deallocation/?ref=ml_lbp
    /// In: start index of allocation
    fn dealloc(&mut self, start: usize) -> OctaResult<()> {
        // If no such starting address available
        let size = self.mp.remove(&start);
        if size.is_none() {
            bail!("Invalid start");
        }
        let size = size.unwrap().to_owned();
        let n = calc_n(size).max(self.min_n) - self.min_n;
        
        let space = (start, start + usize::pow(2, n as u32) - 1);

        debug!("Memory block from {} to {} freed", space.0, space.1);
        
        self.free_list[n].push(space);

        // Calculate buddy number
        let buddy_number = start / size;
        let buddy_address = if buddy_number % 2 != 0 {
            start - usize::pow(2, n as u32)
        } else {
            start + usize::pow(2, n as u32)
        };

        for i in 0..self.free_list[n].len() {
            // If buddy found and is also free
            if self.free_list[n][i].0 == buddy_address {
                // Now merge the buddies to make
                // them one large free memory block
                if buddy_number % 2 == 0 {
                    self.free_list[n + 1].push((start, start + 2 * usize::pow(2, n as u32) - 1));

                    #[cfg(test)]
                    println!(
                        "Coalescing of blocks starting at {} and {} was done",
                        start, buddy_address
                    );
                } else {
                    self.free_list[n + 1].push((
                        buddy_address,
                        buddy_address + 2 * usize::pow(2, n as u32) - 1,
                    ));

                    #[cfg(test)]
                    println!(
                        "Coalescing of blocks starting at {} and {} was done",
                        buddy_address, start
                    );
                }
                self.free_list[n].remove(i);
                let last = self.free_list[n].len() - 1;
                self.free_list[n].remove(last);

                break;
            }
        }

        Ok(())
    }
}

fn calc_n(size: usize) -> usize {
    f32::ceil(f32::ln(size as f32) / f32::ln(2.0)) as usize
}

impl Debug for BuddyBufferAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuddyBufferAllocator")
            .field("inner", &self.inner.lock())
            .finish()
    }
}

impl Debug for BuddyAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuddyAllocation")
            .field("inner", &())
            .field("start", &self.start)
            .field("size", &self.size)
            .finish()
    }
}

mod test {
    use crate::multi_data_buffer::buddy_buffer_allocator::BuddyAllocator;

    #[test]
    fn test_alloc() {
        let mut buddy = BuddyAllocator::new(128, 0);
        buddy.alloc(32).unwrap();
        buddy.alloc(7).unwrap();
        buddy.alloc(64).unwrap();
        assert!(buddy.alloc(56).is_err());
    }

    #[test]
    fn test_dealloc() {
        let mut buddy = BuddyAllocator::new(128, 0);
        buddy.alloc(16).unwrap();
        buddy.alloc(16).unwrap();
        buddy.alloc(16).unwrap();
        buddy.alloc(16).unwrap();
        buddy.dealloc(0).unwrap();
        assert!(buddy.dealloc(9).is_err());
        buddy.dealloc(32).unwrap();
        buddy.dealloc(16).unwrap();
    }
}
