use std::{collections::VecDeque, iter, mem};

use itertools::Itertools;
use octa_force::log::debug;
use smallvec::SmallVec;

use crate::{model::{data_types::{data_type::TemplateValue, number::ValueIndexNumber, position::{ValueIndexPosition, ValueIndexPosition2D, ValueIndexPosition3D}, position_pair_set::ValueIndexPositionSet, position_space::ValueIndexPositionSpace, volume::ValueIndexVolume}, template::{TEMPLATE_INDEX_NONE, Template, TemplateIndex, nodes::TemplateNode, value::ValueIndex}}, util::{default_types::T, number::Nu, vector::Ve}};

use super::{collapser::{CollapseNodeKey, Collapser, UpdateDefinesOperation}, pending_operations::PendingOperations};

#[derive(Debug, Clone, Copy)]
pub struct MatchValueData<'a> {
    pub template: &'a Template,
    pub other_template: &'a Template,
    pub matched_template_indecies: &'a Vec<TemplateIndex>,
}

impl Collapser {
    
    pub fn template_changed(&mut self, new_template: Template) {
        self.pending.template_changed(new_template.max_level);

        let mut new_nodes_per_template_index = vec![SmallVec::new(); new_template.nodes.len()];
        
        // [new] = old
        let mut matched_template_indecies = vec![TEMPLATE_INDEX_NONE; new_template.nodes.len()];
        matched_template_indecies[0] = 0;
      
        let mut to_match: Vec<Vec<(TemplateIndex, TemplateIndex)>> = iter::repeat_with(|| {Vec::new()})
            .take(self.template.max_level +1)
            .collect();

        // Push root node
        let mut left_new_children = vec![vec![]; new_template.nodes.len()];
        left_new_children[0] = new_template.nodes[0].creates.iter()
            .map(|c| c.to_create)
            .enumerate()
            .collect();

        for old_child_index in self.template.nodes[0].creates.iter().map(|c| c.to_create) {
            let old_child = &self.template.nodes[old_child_index];
            to_match[old_child.level].push((old_child_index, 0));
        }

        mem::swap(&mut new_nodes_per_template_index[0], 
                    &mut self.nodes_per_template_index[0]);


        let mut min_to_match_level = 0;

        while min_to_match_level <= self.template.max_level {
            if let Some((old_template_index, new_parent_template_index)) = to_match[min_to_match_level].pop() {
                let old_tempalte_node = &self.template.nodes[old_template_index]; 
                let old_value = &self.template.values[old_tempalte_node.value_index];

                debug!("Searching old: {}", old_template_index);

                let new_match_index = left_new_children[new_parent_template_index].iter()
                    .map(|(_, i)| *i)
                    .enumerate()
                    .find(|(_, i)| {
                        debug!("testing new: {i}");

                        let new_child = &new_template.nodes[*i];
                        let new_value = &new_template.values[new_child.value_index];

                        let v = old_value.match_template_value(new_value, MatchValueData { 
                            template: &self.template, 
                            other_template: &new_template, 
                            matched_template_indecies: &matched_template_indecies
                        });
                        
                        debug!("res: {v}");

                        v
                    });

                if let Some((i, new_match_index)) = new_match_index {
                    debug!("new: {new_match_index} found!");

                    left_new_children[new_parent_template_index].swap_remove(i);
                    matched_template_indecies[new_match_index] = old_template_index;

                    left_new_children[new_match_index] = new_template.nodes[new_match_index].creates.iter()
                        .map(|c| c.to_create)
                        .enumerate()
                        .collect();
                    
                    for old_child_index in new_template.nodes[old_template_index].creates.iter().map(|c| c.to_create) {
                        let old_child = &new_template.nodes[old_child_index];
                        to_match[old_child.level].push((old_child_index, new_match_index));
                    }

                    mem::swap(&mut new_nodes_per_template_index[new_match_index], 
                    &mut self.nodes_per_template_index[old_template_index]);

                } else {
                    debug!("not found!");

                    for index in self.nodes_per_template_index[old_template_index].to_owned() {
                        self.delete_node(index);
                    }
                }
  
            } else {
                min_to_match_level += 1;
            }
        }

        for (new_parent_index, left_new_children) in left_new_children.into_iter().enumerate() {
            for (creates_index, left_new) in left_new_children {
                let level = new_template.nodes[left_new].level;

                debug!("adding new: {left_new} parent: {new_parent_index}");
                
                for index in new_nodes_per_template_index[new_parent_index].iter() {
                    self.pending.push_create_defined(level, UpdateDefinesOperation { 
                        template_index: left_new,
                        parent_index: *index, 
                        creates_index: creates_index,
                    });
                } 
            }
        }

        self.template = new_template;
        self.nodes_per_template_index = new_nodes_per_template_index;
    }
}

impl<'a> MatchValueData<'a> {
    pub fn match_two_numbers(self, n1: ValueIndexNumber, n2: ValueIndexNumber) -> bool {
        self.template.get_number_value(n1)
            .match_value(
                self.other_template.get_number_value(n2), 
                self
            )
    }

    pub fn match_two_positions2d(self, p1: ValueIndexPosition2D, p2: ValueIndexPosition2D) -> bool {
        self.template.get_position2d_value(p1)
            .match_value(
                self.other_template.get_position2d_value(p2), 
                self
            )
    }

    pub fn match_two_positions3d(self, p1: ValueIndexPosition3D, p2: ValueIndexPosition3D) -> bool {
        self.template.get_position3d_value(p1)
            .match_value(
                self.other_template.get_position3d_value(p2), 
                self
            )
    }

    pub fn match_two_positions<V: Ve<T, D>, const D: usize>(self, p1: ValueIndexPosition, p2: ValueIndexPosition) -> bool {
        self.template.get_position_value::<V, D>(p1)
            .match_value(
                self.other_template.get_position_value::<V, D>(p2), 
                self
            )
    }

    pub fn match_two_positions_check(self, p1: ValueIndexPosition, p2: ValueIndexPosition) -> bool {
        match &self.template.values[p1] {
            TemplateValue::Position2D(p1) 
            => match &self.other_template.values[p2] {
                TemplateValue::Position2D(p2) => p1.match_value(p2, self),
                _ => false,
            },
            TemplateValue::Position3D(p1) 
            => match &self.other_template.values[p2] {
                TemplateValue::Position3D(p2) => p1.match_value(p2, self),
                _ => false,
            },
            _ => unreachable!(),
        }

    }

    pub fn match_two_volumes(self, p1: ValueIndexVolume, p2: ValueIndexVolume) -> bool {
        self.template.get_volume_value(p1)
            .match_value(
                self.other_template.get_volume_value(p2), 
                self
            )
    }

    pub fn match_two_position_set(self, p1: ValueIndexPositionSet, p2: ValueIndexPositionSet) -> bool {
        self.template.get_position_set_value(p1)
            .match_value(
                self.other_template.get_position_set_value(p2), 
                self
            )
    }

    pub fn match_two_position_space(self, p1: ValueIndexPositionSpace, p2: ValueIndexPositionSpace) -> bool {
        self.template.get_position_space_value(p1)
            .match_value(
                self.other_template.get_position_space_value(p2), 
                self
            )
    }
}



