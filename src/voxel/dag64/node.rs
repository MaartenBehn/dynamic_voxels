use crate::{gi::probe_pool::GI_PROBE_INDEX_NONE, util::math::count_ones_variable};


#[repr(C, packed)]
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct VoxelDAG64Node {
    pub is_leaf_and_index: u32,
    pub gi_index: u32,
    pub pop_mask: u64,
}

impl VoxelDAG64Node {
    pub fn new(is_leaf: bool, index: u32, pop_mask: u64, gi_index: u32) -> Self {
        Self {
            is_leaf_and_index: (index << 1) | (is_leaf as u32),
            pop_mask,
            gi_index
        }
    }
   
    pub fn single(is_leaf: bool, index: u32, pop_mask: u64) -> Self {
        Self {
            is_leaf_and_index: (index << 1) | (is_leaf as u32),
            pop_mask,
            gi_index: GI_PROBE_INDEX_NONE,
        }
    }

    pub fn is_leaf(&self) -> bool {
        (self.is_leaf_and_index & 1) == 1
    }

    pub fn index(&self) -> u32 {
        self.is_leaf_and_index >> 1
    }

    pub fn is_occupied(&self, index: u32) -> bool {
        self.pop_mask >> index & 1 == 1
    }

    pub fn get_index_for_child(&self, child: u32) -> Option<u32> {
        Some(self.index() + self.get_index_in_children(child)?)
    }

    pub fn get_index_in_children(&self, index: u32) -> Option<u32> {
        if !self.is_occupied(index) {
            return None;
        }

        Some(count_ones_variable(self.pop_mask, index))
    }

    pub fn get_index_in_children_unchecked(&self, index: u32) -> u32 {
        count_ones_variable(self.pop_mask, index)
    }


    pub fn range(&self) -> std::ops::Range<usize> {
        self.index() as usize..self.index() as usize + self.pop_mask.count_ones() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.pop_mask == 0
    }

    pub fn check_empty(&self) -> Option<Self> {
        Some(*self).filter(|node| !node.is_empty())
    }
}

