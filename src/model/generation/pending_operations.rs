use std::iter;

use octa_force::log::debug;

use super::collapse::{CollapseNodeKey};


#[derive(Debug, Clone)]
pub struct PendingOperations {
    pending_per_level: Vec<Vec<CollapseNodeKey>>,
    min_with_value: usize,
}


impl PendingOperations {
    pub fn new(max_level: usize) -> Self {
        Self {
            pending_per_level: iter::repeat_with(|| {vec![]}).take(max_level).collect(),
            min_with_value: max_level,
        }
    }

    pub fn push(&mut self, level: usize, index: CollapseNodeKey) {
        self.pending_per_level[level - 1].push(index);
        self.min_with_value = self.min_with_value.min(level - 1);
    }

    pub fn pop(&mut self) -> Option<CollapseNodeKey> {
        let res = self.pending_per_level[self.min_with_value].pop();
        if res.is_none() {
            return None;
        }

        self.find_next_higher_filled_level();

        res
    }

    pub fn delete(&mut self, level: usize, index: CollapseNodeKey) {
        let to_delete: Vec<_> = self.pending_per_level[level - 1].iter()
            .enumerate()
            .filter(|(_, key)| **key == index)
            .map(|(i, _)| i)
            .collect();

        if to_delete.is_empty() {
            return;
        }

        for i in to_delete {
            self.pending_per_level[level - 1].swap_remove(i);
        }

        self.find_next_higher_filled_level();
    } 

    fn find_next_higher_filled_level(&mut self) {
        for i in self.min_with_value..self.pending_per_level.len() {
            if !self.pending_per_level[i].is_empty() {
                self.min_with_value = i;
                return;
            }
        }
        self.min_with_value = self.pending_per_level.len() -1;
    }
}
