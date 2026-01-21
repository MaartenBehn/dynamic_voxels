use std::mem;

use smallvec::SmallVec;

use crate::{model::{composer::{graph::ComposerNodeFlags}, template::{nodes::{TemplateNode}, update::TemplateNodeUpdate, Template}}};

use super::{collapser::{CollapseNodeKey, Collapser, UpdateDefinesOperation}, pending_operations::PendingOperations};


impl Collapser {
    
    pub fn template_changed(
        &mut self, 
        template: &Template,
        old_template: &Template,
        updates: Vec<TemplateNodeUpdate>,
    ) {
        self.pending.template_changed(template.max_level);

        let mut new_nodes_per_template_index = vec![SmallVec::new(); template.nodes.len()]; 
        for update in updates {
            match update {
                TemplateNodeUpdate::Delete(template_index) => {
                    let keys = self.nodes.keys().collect::<Vec<_>>(); 
                    for key in keys {
                        let node = &self.nodes[key];
                        if node.template_index == template_index {
                            self.delete_node(key, old_template);
                        }
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
                TemplateNodeUpdate::Unchanged { old, new } => {
                    for index in self.nodes_per_template_index[old].iter() {
                        self.nodes[*index].template_index = new;
                    }

                    mem::swap(&mut new_nodes_per_template_index[new], &mut self.nodes_per_template_index[old]);
                },
                TemplateNodeUpdate::Changed { old, new, level } => {
                    for index in self.nodes_per_template_index[old].iter() {
                        self.nodes[*index].template_index = new;
                        self.pending.push_collpase(level, *index);
                    }

                    mem::swap(&mut new_nodes_per_template_index[new], &mut self.nodes_per_template_index[old]);
                },
            }
        }

        for (i, per) in self.nodes_per_template_index.iter_mut().enumerate() {
            let new = &mut new_nodes_per_template_index[i];

            if !per.is_empty() && new.is_empty() {
                mem::swap(per, new);
            } 
        }

        self.nodes_per_template_index = new_nodes_per_template_index;
    }
} 

