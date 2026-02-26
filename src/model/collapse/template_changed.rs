use std::mem;

use itertools::Itertools;
use smallvec::SmallVec;

use crate::model::template::{Template, TemplateIndex, nodes::TemplateNode, update::TemplateNodeUpdate, value::ValueIndex};

use super::{collapser::{CollapseNodeKey, Collapser, UpdateDefinesOperation}, pending_operations::PendingOperations};

pub struct MatchValueData<'a> {
    pub template: &'a Template,
    pub other_template: &'a Template,
    pub matched_template_indecies: Vec<TemplateIndex>,
}

impl Collapser {
    
    pub fn template_changed(
        &mut self, 
        new_template: Template,
        updates: Vec<TemplateNodeUpdate>,
    ) {
        self.pending.template_changed(new_template.max_level);

        let mut new_nodes_per_template_index = vec![SmallVec::new(); new_template.nodes.len()]; 

        for update in updates {
            match update {
                TemplateNodeUpdate::Delete(template_index) => {
                    for index in self.nodes_per_template_index[template_index].iter().copied().collect_vec() {
                        self.delete_node(index);
                    }
                },
                TemplateNodeUpdate::New{ new, parent, creates_index, new_level } => { 
                    for index in self.nodes_per_template_index[parent].iter() {
                        self.pending.push_create_defined(new_level, UpdateDefinesOperation { 
                            template_index: new,
                            parent_index: *index, 
                            creates_index,
                        });
                    }
                },
                TemplateNodeUpdate::Changed { old, new, level } => {
                    for index in self.nodes_per_template_index[old].iter() {
                        self.nodes[*index].template_index = new;
                        self.pending.push_collpase(level, *index);
                    }

                    mem::swap(&mut new_nodes_per_template_index[new], &mut self.nodes_per_template_index[old]);
                },
                TemplateNodeUpdate::UpdateIndex { old, new } => {
                    for index in self.nodes_per_template_index[old].iter() {
                        self.nodes[*index].template_index = new;
                    }

                    mem::swap(&mut new_nodes_per_template_index[new], &mut self.nodes_per_template_index[old]);
                },
                TemplateNodeUpdate::None(template_index) => {
                    mem::swap(&mut new_nodes_per_template_index[template_index], &mut self.nodes_per_template_index[template_index]);
                }
            }
        }

        self.template = new_template;
        self.nodes_per_template_index = new_nodes_per_template_index;
    }

    pub fn match_template_node(
        &mut self, 
        old_template_index: TemplateIndex, 
        new_template_index: TemplateIndex, 
        new_template: &Template, 
        new_nodes_per_template_index: &mut Vec<SmallVec<[CollapseNodeKey; 4]>>,
    ) {

        let old_template_node = &self.template.nodes[old_template_index];
        let new_tempalte_node = &new_template.nodes[new_template_index]; 

        let old_value = &self.template.values[old_template_node.value_index];
        let new_value = &new_template.values[new_tempalte_node.value_index];

        let value_same = old_value.match_value(new_value, MatchValueData { 
            template: &self.template, 
            other_template: new_template, 
            matched_template_indecies: vec![]
        });

        if !value_same {
            for index in self.nodes_per_template_index[old_template_index].iter() {
                self.nodes[*index].template_index = new_template_index;
                self.pending.push_collpase(new_tempalte_node.level, *index);
            }
        } else if old_template_index != new_template_index {
            for index in self.nodes_per_template_index[old_template_index].iter() {
                self.nodes[*index].template_index = new_template_index;
            }
        }

        let mut matched_creates = 
        let mut old_creates = old_template_node.creates.to_owned();
        for (i, new_creates) in new_tempalte_node.creates.iter().enumerate() {

            let new_child = &new_template.nodes[new_creates.to_create];
            let new_child_value = &new_template.values[new_child.value_index];

            let matched_old_creates_index = old_creates.iter().position(|old_create| {

                let old_child = &self.template.nodes[old_create.to_create];
                let old_child_value = &self.template.values[old_child.value_index];

                old_child_value.match_value(new_child_value, MatchValueData { 
                    template: &self.template, 
                    other_template: new_template, 
                    matched_template_indecies: vec![]
                })
            });

            if let Some(matched_old_creates_index) = matched_old_creates_index {
                let matched_old_creates = old_creates.swap_remove(matched_old_creates_index); 
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
    }
}



