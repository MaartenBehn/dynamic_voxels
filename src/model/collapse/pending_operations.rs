use core::fmt;
use std::{collections::VecDeque, iter, mem};

use octa_force::log::debug;

use super::collapser::{CollapseNodeKey, UpdateDefinesOperation};

#[derive(Debug, Clone)]
pub struct PendingOperations {
    pub pending_collapse: Vec<VecDeque<CollapseNodeKey>>,
    pub pending_create_defined: Vec<VecDeque<UpdateDefinesOperation>>,
    pub pending_next_collapse: Vec<VecDeque<CollapseNodeKey>>,
    pub min_with_value: usize,
    pub next_min_with_value: usize,
}

pub enum PendingOperationsRes {
    Collapse(CollapseNodeKey),
    CreateDefined(UpdateDefinesOperation),
    Retry,
    Empty
}

impl PendingOperations {
    pub fn empty() -> Self {
        Self {
            min_with_value: 0,
            next_min_with_value: 0,
            pending_collapse: vec![],
            pending_create_defined: vec![],
            pending_next_collapse: vec![],
        }
    }

    pub fn new(max_level: usize) -> Self {
        Self {
            pending_collapse: iter::repeat_with(|| {VecDeque::new()}).take(max_level).collect(),
            pending_create_defined: iter::repeat_with(|| {VecDeque::new()}).take(max_level).collect(),
            pending_next_collapse: iter::repeat_with(|| {VecDeque::new()}).take(max_level).collect(),
            min_with_value: max_level -1,
            next_min_with_value: max_level -1,
        }
    }

    pub fn template_changed(&mut self, max_level: usize) {
        
        for list in self.pending_create_defined.iter_mut() {
            list.clear();
        }

        for list in self.pending_collapse.iter_mut() {
            list.clear();
        }

        for list in self.pending_next_collapse.iter_mut() {
            list.clear();
        }
        
        let new_min_with_value = max_level -1; 
        if self.min_with_value != new_min_with_value {
            self.pending_collapse.resize(max_level, VecDeque::new());
            self.pending_create_defined.resize(max_level, VecDeque::new());
            self.pending_next_collapse.resize(max_level, VecDeque::new());
            self.min_with_value = new_min_with_value;
            self.next_min_with_value = new_min_with_value;
        }
    }

    pub fn push_collpase(&mut self, level: usize, value: CollapseNodeKey) {
        if self.pending_collapse[level - 1].contains(&value) {
            return;
        }   

        self.pending_collapse[level - 1].push_back(value);
        self.min_with_value = self.min_with_value.min(level - 1);
    }

    pub fn push_later_collpase(&mut self, level: usize, value: CollapseNodeKey) {
        if self.pending_next_collapse[level - 1].contains(&value) {
            return;
        }

        self.pending_next_collapse[level - 1].push_back(value);
        self.next_min_with_value = self.next_min_with_value.min(level - 1);
    }

    pub fn push_create_defined(&mut self, level: usize, value: UpdateDefinesOperation) {
        if self.pending_create_defined[level - 1].contains(&value) {
            return;
        }

        self.pending_create_defined[level - 1].push_back(value);
        self.min_with_value = self.min_with_value.min(level - 1);
    }

    pub fn pop(&mut self) -> PendingOperationsRes {

        loop {
            let res = self.pending_create_defined[self.min_with_value].pop_front();
            if let Some(value) = res {
                return PendingOperationsRes::CreateDefined(value);
            }

            let res = self.pending_collapse[self.min_with_value].pop_front();
            if let Some(value) = res {
                return PendingOperationsRes::Collapse(value);
            }

            if self.min_with_value >= self.pending_collapse.len() -1 {
                break;
            }

            self.update_min_value();
        }

        if self.next_min_with_value != self.pending_collapse.len() -1 {
            mem::swap(&mut self.pending_collapse, &mut self.pending_next_collapse);
            mem::swap(&mut self.min_with_value, &mut self.next_min_with_value);
            return PendingOperationsRes::Retry;
        }
        
        PendingOperationsRes::Empty
    }

    pub fn delete_collapse(&mut self, level: usize, value: CollapseNodeKey) {
        let list = &mut self.pending_collapse[level -1];
        for i in (0..list.len()).rev() {
            if list[i] == value {
                list.swap_remove_back(i);
            }
        }
        
        let list = &mut self.pending_next_collapse[level -1];
        for i in (0..list.len()).rev() {
            if list[i] == value {
                list.swap_remove_back(i);
            }
        }
    }

    pub fn delete_create_defined(&mut self, parent_index: CollapseNodeKey) {
        for list in self.pending_create_defined.iter_mut() {

            for i in (0..list.len()).rev() {
                if list[i].parent_index == parent_index {
                    list.swap_remove_back(i);
                }
            }
        }
    } 

    fn update_min_value(&mut self) {
        for i in 0..self.pending_collapse.len() {
            if !self.pending_collapse[i].is_empty() || !self.pending_create_defined[i].is_empty() {
                self.min_with_value = i;
                return;
            }
        }
        self.min_with_value = self.pending_collapse.len() -1;
    }

    fn update_next_min_value(&mut self) {
       for i in 0..self.pending_next_collapse.len() {
            if !self.pending_next_collapse[i].is_empty() {
                self.next_min_with_value = i;
                return;
            }
        }
        self.next_min_with_value = self.pending_next_collapse.len() -1;
    }
}

impl Default for PendingOperations {
    fn default() -> Self {
        Self::empty()
    }
}
