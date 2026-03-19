use octa_force::anyhow::bail;
use octa_force::log::{debug, error, trace};
use octa_force::OctaResult;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::sync::{Arc};

#[derive(Clone)]
pub struct SharedBuddyAllocator {
    inner: Arc<Mutex<BuddyAllocator>>,
}

pub struct BuddyAllocation {
    inner: SharedBuddyAllocator,
    alloc: ManualBuddyAllocation
}

#[derive(Clone, Copy)]
pub struct ManualBuddyAllocation {
    start: usize,
    size: usize,
}

#[derive(Debug, PartialEq)]
pub struct BuddyAllocator {
    free_list: Vec<Vec<(usize, usize)>>,
    mp: HashMap<usize, usize>,
    size: usize,
    min_n: usize,
}

impl SharedBuddyAllocator {
    pub fn new(size: usize, min_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BuddyAllocator::new(size, min_size))),
        }
    }

    pub fn alloc(&mut self, size: usize) -> OctaResult<BuddyAllocation> {
        let alloc = self.inner.lock().alloc(size)?;

        Ok(BuddyAllocation {
            inner: self.clone(),
            alloc,
        })
    }
}

impl BuddyAllocation {
    pub fn start(&self) -> usize { self.alloc.start() }
    pub fn size(&self) -> usize { self.alloc.size() }
}

impl ManualBuddyAllocation {
    pub fn start(&self) -> usize { self.start }
    pub fn size(&self) -> usize { self.size }
}

impl Drop for BuddyAllocation {
    fn drop(&mut self) {
        let res = self.inner.inner.lock().dealloc(self.alloc);
        if let Err(e) = res {
            error!("Buddy Allocation Drop failed with: {e}")
        }
    }
}

impl BuddyAllocator {
    pub fn new(size: usize, min_size: usize) -> Self {
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

    pub fn clear(&mut self) {
        self.free_list.iter_mut().for_each(|l| l.clear()); 
        self.free_list.last_mut().unwrap().push((0, self.size - 1));
        self.mp.clear();
    }

    // From https://www.geeksforgeeks.org/buddy-memory-allocation-program-set-1-allocation/
    /// In: size in byte
    /// Out: start index and size of allocation
    pub fn alloc(&mut self, size: usize) -> OctaResult<ManualBuddyAllocation> {
        // Calculate index in free list
        // to search for block if available
        let n = calc_n(size).max(self.min_n) - self.min_n;

        if n >= self.free_list.len() {
            bail!("Requested to large allocation");
        }

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
        //debug!("Memory from {} to {} for {} bytes allocated", space.0, space.1, size);

        let size = space.1 - space.0 + 1;
        self.mp.insert(space.0, size);

        Ok(ManualBuddyAllocation { 
            start: space.0, 
            size
        })
    }

    // From https://www.geeksforgeeks.org/buddy-memory-allocation-program-set-2-deallocation/?ref=ml_lbp
    /// In: start index of allocation
    pub fn dealloc(&mut self, alloc: ManualBuddyAllocation) -> OctaResult<()> {
        // If no such starting address available
        let size = self.mp.remove(&alloc.start);
        if size.is_none() {
            bail!("Invalid start");
        }
        let size = size.unwrap().to_owned();
        let full_n = calc_n(size);
        let n = full_n.max(self.min_n) - self.min_n;
        
        let space = (alloc.start, alloc.start + usize::pow(2, full_n as u32) - 1);

        //debug!("Memory block from {} to {} freed", space.0, space.1);
        
        self.free_list[n].push(space);

        // Calculate buddy number
        let buddy_number = alloc.start / size;
        let buddy_address = if buddy_number % 2 != 0 {
            alloc.start - usize::pow(2, full_n as u32)
        } else {
            alloc.start + usize::pow(2, full_n as u32)
        };

        for i in 0..self.free_list[n].len() {
            // If buddy found and is also free
            if self.free_list[n][i].0 == buddy_address {
                // Now merge the buddies to make
                // them one large free memory block
                if buddy_number % 2 == 0 {
                    self.free_list[n + 1].push((alloc.start, alloc.start + 2 * usize::pow(2, full_n as u32) - 1));

                    #[cfg(test)]
                    println!(
                        "Coalescing of blocks starting at {} and {} was done",
                        alloc.start, buddy_address
                    );
                } else {
                    self.free_list[n + 1].push((
                        buddy_address,
                        buddy_address + 2 * usize::pow(2, full_n as u32) - 1,
                    ));

                    #[cfg(test)]
                    println!(
                        "Coalescing of blocks starting at {} and {} was done",
                        buddy_address, alloc.start
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

    pub fn size(&self) -> usize {
        self.size
    }
}

fn calc_n(size: usize) -> usize {
    f32::ceil(f32::ln(size as f32) / f32::ln(2.0)) as usize
}

impl Debug for SharedBuddyAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedBuddyBufferAllocator")
            .field("inner", &self.inner.lock())
            .finish()
    }
}

impl Debug for ManualBuddyAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuddyAllocation")
            .field("start", &self.start())
            .field("size", &self.size())
            .finish()
    }
}

impl Debug for BuddyAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuddyAllocation")
            .field("start", &self.start())
            .field("size", &self.size())
            .finish()
    }
}

impl PartialEq for ManualBuddyAllocation {
    fn eq(&self, other: &Self) -> bool {
        self.start() == other.start() && self.size() == other.size()
    }
}

impl PartialEq for BuddyAllocation {
    fn eq(&self, other: &Self) -> bool {
        self.alloc == other.alloc
    }
}

impl PartialEq for SharedBuddyAllocator {
    fn eq(&self, other: &Self) -> bool {
        let a = self.inner.lock();
        let b = other.inner.lock();

        a.size == b.size
        && a.mp == b.mp
        && a.free_list == b.free_list
        && a.min_n == b.min_n
    }
}

mod test {
    use super::BuddyAllocator;

    /*
    #[test]
    fn test_same_size_alloc() {
        let mut buddy = BuddyAllocator::new(32, 0);
        assert_eq!(buddy.alloc(8).unwrap(), (0, 8));
        assert_eq!(buddy.alloc(8).unwrap(), (8, 8));
        assert_eq!(buddy.alloc(8).unwrap(), (16, 8));
        assert_eq!(buddy.alloc(8).unwrap(), (24, 8));
        assert!(buddy.alloc(8).is_err());
    }

    #[test]
    fn test_differnet_size_alloc() {
        let mut buddy = BuddyAllocator::new(32, 0);
        assert_eq!(buddy.alloc(8).unwrap(), (0, 8));
        assert_eq!(buddy.alloc(4).unwrap(), (8, 4));
        assert_eq!(buddy.alloc(8).unwrap(), (16, 8));
        assert_eq!(buddy.alloc(4).unwrap(), (12, 4));
        assert_eq!(buddy.alloc(8).unwrap(), (24, 8));
        assert!(buddy.alloc(1).is_err());
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
    */
}
