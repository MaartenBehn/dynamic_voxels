use std::collections::HashMap;
use std::iter;
use octa_force::anyhow::bail;
use octa_force::OctaResult;

pub struct BuddyBufferAllocator {
    free_list: Vec<Vec<(usize, usize)>>,
    mp: HashMap<usize, usize>,
}

impl BuddyBufferAllocator {
    pub fn new(size: usize) -> Self {
        let n = f32::ceil(f32::ln(size as f32) / f32::ln(2.0)) as usize;

        let free_list: Vec<_> = iter::repeat(vec![])
            .take(n + 1)
            .chain([vec![(0, size - 1)]].into_iter())
            .collect();
        
        BuddyBufferAllocator {
            free_list,
            mp: Default::default(),
        }
    }

    // From https://www.geeksforgeeks.org/buddy-memory-allocation-program-set-1-allocation/
    pub fn alloc(&mut self, size: usize) -> OctaResult<(usize, usize)> {
        
        // Calculate index in free list 
        // to search for block if available
        let n = f32::ceil(f32::ln(size as f32) / f32::ln(2.0)) as usize;

        let space = if !self.free_list[n].is_empty() {
            self.free_list[n].remove(0)
        }
        else
        {
            let found = self.free_list[(n+1)..]
                .iter_mut()
                .enumerate()
                .find_map(|(i, free)| { 
                    if !free.is_empty() { 
                        Some((i + n, free.remove(0))) 
                    } else { 
                        None 
                    } });

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

        #[cfg(test)]
        println!("Memory from {} to {} allocated", space.0, space.1);

        // map starting address with 
        // size to make deallocating easy
        self.mp.insert(space.0, space.1 - space.0 + 1);
        
        Ok(space)
    }

    // From https://www.geeksforgeeks.org/buddy-memory-allocation-program-set-2-deallocation/?ref=ml_lbp
    pub fn dealloc(&mut self, start: usize) -> OctaResult<()> {
        // If no such starting address available
        let size = self.mp.remove(&start);
        if size.is_none() {
            bail!("Invalid start");
        }
        let size = size.unwrap().to_owned();

        let n = f32::ceil(f32::ln(size as f32) / f32::ln(2.0)) as usize;
        
        let space = (start, start + usize::pow(2, n as u32) - 1);
        
        #[cfg(test)]
        println!("Memory block from {} to {} freed", space.0, space.1);
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
                    println!("Coalescing of blocks starting at {} and {} was done", start, buddy_address);
                }
                else
                {
                    self.free_list[n + 1].push((buddy_address, buddy_address + 2 * usize::pow(2, n as u32) - 1));

                    #[cfg(test)]
                    println!("Coalescing of blocks starting at {} and {} was done", buddy_address, start);
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

mod test {
    use crate::buddy_controller::BuddyBufferAllocator;

    #[test]
    fn test_alloc() {
        let mut buddy = BuddyBufferAllocator::new(128);
        buddy.alloc(32).unwrap();
        buddy.alloc(7).unwrap();
        buddy.alloc(64).unwrap();
        assert!(buddy.alloc(56).is_err());

    }
    
    #[test]
    fn test_dealloc() {
        let mut buddy = BuddyBufferAllocator::new(128);
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