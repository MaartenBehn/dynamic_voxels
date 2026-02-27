use std::{collections::VecDeque, iter, mem};

use itertools::Itertools;
use smallvec::SmallVec;

use crate::model::template::{TEMPLATE_INDEX_NONE, Template, TemplateIndex, nodes::TemplateNode, update::TemplateNodeUpdate, value::ValueIndex};

use super::{collapser::{CollapseNodeKey, Collapser, UpdateDefinesOperation}, pending_operations::PendingOperations};

pub struct MatchValueData<'a> {
    pub template: &'a Template,
    pub other_template: &'a Template,
    pub matched_template_indecies: &'a Vec<TemplateIndex>,
}

impl Collapser {
    
    pub fn template_changed(&mut self, new_template: Template) {
        self.pending.template_changed(new_template.max_level);

        let mut new_nodes_per_template_index = vec![SmallVec::new(); new_template.nodes.len()]; 
        let mut matched_template_indecies = vec![TEMPLATE_INDEX_NONE; self.template.nodes.len()];
      
        let mut to_match: Vec<Vec<(TemplateIndex, TemplateIndex)>> = iter::repeat_with(|| {Vec::new()})
            .take(self.template.max_level)
            .collect();

        to_match[0].push((0, 0));
        let mut min_to_match_level = 0;

        while min_to_match_level < self.template.max_level {
            if let Some((old_template_index, new_template_index)) = to_match[min_to_match_level].pop() {
                matched_template_indecies.push(new_template_index);

                let old_template_node = &self.template.nodes[old_template_index];
                let new_tempalte_node = &new_template.nodes[new_template_index]; 
           
                if old_template_index != new_template_index {
                    for index in self.nodes_per_template_index[old_template_index].iter() {
                        self.nodes[*index].template_index = new_template_index;
                    }
                }

                let mut old_creates = old_template_node.creates.to_owned();
                for (i, new_creates) in new_tempalte_node.creates.iter().enumerate() {

                    let new_child = &new_template.nodes[new_creates.to_create];
                    let new_child_value = &new_template.values[new_child.value_index];

                    let matched_old_child = old_creates.iter()
                        .map(|old_create| &self.template.nodes[old_create.to_create])
                        .enumerate()
                        .find(|(_, old_child)| {

                        let old_child_value = &self.template.values[old_child.value_index];

                        old_child_value.match_value(new_child_value, MatchValueData { 
                            template: &self.template, 
                            other_template: &new_template, 
                            matched_template_indecies: &matched_template_indecies
                        })
                    });

                    if let Some((old_creates_index, old_child)) = matched_old_child {
                        old_creates.swap_remove(old_creates_index);

                        to_match[old_child.level].push((old_child.index, new_child.index));
                        min_to_match_level = min_to_match_level.min(old_child.level);

                    } else {
                        for index in self.nodes_per_template_index[old_template_index].iter() {
                            self.pending.push_create_defined(new_child.level, UpdateDefinesOperation { 
                                template_index: new_creates.to_create,
                                parent_index: *index, 
                                creates_index: i,
                            });
                        }
                    }
                }

                for old_create in old_creates {
                    for index in self.nodes_per_template_index[old_create.to_create].iter().copied().collect_vec() {
                        self.delete_node(index);
                    } 
                }

                mem::swap(&mut new_nodes_per_template_index[new_template_index], &mut self.nodes_per_template_index[old_template_index]);

            } else {
                min_to_match_level += 1;
            }
        }
 
        self.template = new_template;
        self.nodes_per_template_index = new_nodes_per_template_index;
    }
}



