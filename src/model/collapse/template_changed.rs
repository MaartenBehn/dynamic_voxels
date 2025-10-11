use crate::{model::composer::{build::BS, template::ComposeTemplate}, util::{number::Nu, vector::Ve}};

use super::{collapser::{CollapseNodeKey, Collapser}, pending_operations::PendingOperations};


impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    
    pub fn template_changed(&mut self, template: &ComposeTemplate<V2, V3, T, B>, state: &mut B) {
        self.pending.template_changed(template.max_level);

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

