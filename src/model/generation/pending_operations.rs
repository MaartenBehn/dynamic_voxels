use core::fmt;
use std::{collections::VecDeque, iter};

use octa_force::log::debug;

use super::collapse::{CollapseNodeKey, CreateDefinesOperation};


#[derive(Debug, Clone)]
pub struct PendingOperations {
    pub pending_per_level: Vec<(VecDeque<CollapseNodeKey>, VecDeque<CreateDefinesOperation>)>,
    pub min_with_value: usize,
}

pub enum PendingOperationsRes {
    Collapse(CollapseNodeKey),
    CreateDefined(CreateDefinesOperation),
    Empty
}

impl PendingOperations {
    pub fn new(max_level: usize) -> Self {
        Self {
            pending_per_level: iter::repeat_with(|| {(VecDeque::new(), VecDeque::new())}).take(max_level).collect(),
            min_with_value: max_level -1,
        }
    }

    pub fn push_collpase(&mut self, level: usize, value: CollapseNodeKey) {
        self.pending_per_level[level - 1].0.push_back(value);
        self.min_with_value = self.min_with_value.min(level - 1);
    }

    pub fn push_create_defined(&mut self, level: usize, value: CreateDefinesOperation) {
        self.pending_per_level[level - 1].1.push_back(value);
        self.min_with_value = self.min_with_value.min(level - 1);
    }

    pub fn pop(&mut self) -> PendingOperationsRes {

        loop {
            let res = self.pending_per_level[self.min_with_value].1.pop_front();
            if let Some(value) = res {
                return PendingOperationsRes::CreateDefined(value);
            }

            let res = self.pending_per_level[self.min_with_value].0.pop_front();
            if let Some(value) = res {
                return PendingOperationsRes::Collapse(value);
            }

            if self.min_with_value >= self.pending_per_level.len() -1 {
                break;
            }

            self.find_next_higher_filled_level();
        }
        
        PendingOperationsRes::Empty
    }

    pub fn delete_collapse(&mut self, level: usize, value: CollapseNodeKey) {
        let level = &mut self.pending_per_level[level -1].0;

        for i in (0..level.len()).rev() {
            if level[i] == value {
                level.swap_remove_back(i);
            }
        }
    }

    pub fn delete_create_defined(&mut self, parent_index: CollapseNodeKey) {
        for level in self.pending_per_level.iter_mut()
            .map(|(_, v)| v) {

            for i in (0..level.len()).rev() {
                if level[i].get_parent_index() == parent_index {
                    level.swap_remove_back(i);
                }
            }
        }
    } 

    fn find_next_higher_filled_level(&mut self) {
       self.find_next_filled_level(self.min_with_value); 
    }

    fn find_next_filled_level(&mut self, start: usize) {
        for i in 0..self.pending_per_level.len() {
            if !self.pending_per_level[i].0.is_empty() || !self.pending_per_level[i].1.is_empty() {
                self.min_with_value = i;
                return;
            }
        }
        self.min_with_value = self.pending_per_level.len() -1;
    }
}

impl Default for PendingOperations {
    fn default() -> Self {
        Self { pending_per_level: vec![], min_with_value: 0 }
    }
}
