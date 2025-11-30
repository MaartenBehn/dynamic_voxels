use crate::{model::{composer::{build::BS, graph::ComposerNodeFlags}, template::{nodes::{TemplateNode, UpdateType}, update::TemplateNodeUpdate, ComposeTemplate}}, util::{number::Nu, vector::Ve}};

use super::{collapser::{CollapseNodeKey, Collapser, UpdateDefinesOperation}, pending_operations::PendingOperations};


impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    
    pub fn template_changed(
        &mut self, 
        template: &ComposeTemplate<V2, V3, T, B>,
        old_template: &ComposeTemplate<V2, V3, T, B>,
        updates: Vec<TemplateNodeUpdate>,
        state: &mut B
    ) {
        self.pending.template_changed(template.max_level);

        for update in updates {
            match update {
                TemplateNodeUpdate::Delete(template_index) => {
                    let keys = self.nodes.keys().collect::<Vec<_>>(); 
                    for key in keys {
                        let node = &self.nodes[key];
                        if node.template_index == template_index {
                            self.delete_node(key, old_template, state);
                        }
                    }
                },
                TemplateNodeUpdate::New{ new, parent, creates_index, new_level } => { 
                    for index in self.nodes_per_template_index[parent].iter() {
                        self.pending.push_create_defined(new_level, UpdateDefinesOperation { 
                            template_index: index,
                            parent_index: parent, 
                            creates_index,
                        });
                    }
                },
                TemplateNodeUpdate::Unchanged { old, new } => todo!(),
                TemplateNodeUpdate::Changed { old, new } => todo!(),
            }
        }
 
        // TODO only recreate what changed
        while self.nodes.len() > 1 {

            let index = self.nodes.keys().skip(1).next().unwrap();
            self.delete_node(index, template, state);
        }

        self.pending.push_collpase(1, self.get_root_key());
    }
 
    /*
    fn check_node(&mut self, index: CollapseNodeKey, template: &ComposeTemplate<V2, V3, T, B>) {
        let node = &self.nodes[index];
        let template_node = &template.nodes[node.template_index];
    }
    */
} 

