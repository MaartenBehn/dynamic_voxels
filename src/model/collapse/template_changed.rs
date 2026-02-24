use std::mem;

use itertools::Itertools;
use smallvec::SmallVec;

use crate::{model::{template::{nodes::{TemplateNode}, update::TemplateNodeUpdate, Template}}};

use super::{collapser::{CollapseNodeKey, Collapser, UpdateDefinesOperation}, pending_operations::PendingOperations};


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
                TemplateNodeUpdate::None(template_index) => {
                    mem::swap(&mut new_nodes_per_template_index[template_index], &mut self.nodes_per_template_index[template_index]);
                }
            }
        }

        self.template = new_template;
        self.nodes_per_template_index = new_nodes_per_template_index;
    }
} 

