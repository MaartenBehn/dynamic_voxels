use crate::util::count_ones_variable;

#[repr(C, packed)]
#[derive(Default, Debug, Clone, Copy)]
pub struct VoxelDAG64Node {
    pub is_leaf_and_ptr: u32,
    pub pop_mask: u64,
}

impl VoxelDAG64Node {
    fn empty(is_leaf: bool) -> Self {
        Self::new(is_leaf, 0, 0)
    }

    pub fn new(is_leaf: bool, ptr: u32, pop_mask: u64) -> Self {
        Self {
            is_leaf_and_ptr: (ptr << 1) | (is_leaf as u32),
            pop_mask,
        }
    }

    fn is_leaf(&self) -> bool {
        (self.is_leaf_and_ptr & 1) == 1
    }

    fn ptr(&self) -> u32 {
        self.is_leaf_and_ptr >> 1
    }

    fn is_occupied(&self, index: u32) -> bool {
        self.pop_mask >> index & 1 == 1
    }

    fn get_index_for_child(&self, child: u32) -> Option<u32> {
        Some(self.ptr() + self.get_index_in_children(child)?)
    }

    fn get_index_in_children(&self, index: u32) -> Option<u32> {
        if !self.is_occupied(index) {
            return None;
        }

        Some(count_ones_variable(self.pop_mask, index))
    }

    fn range(&self) -> std::ops::Range<usize> {
        self.ptr() as usize..self.ptr() as usize + self.pop_mask.count_ones() as usize
    }

    fn check_empty(&self) -> Option<Self> {
        Some(*self).filter(|node| node.pop_mask != 0)
    }
}
