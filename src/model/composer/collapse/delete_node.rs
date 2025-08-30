use octa_force::{anyhow::{self, ensure, anyhow}, log::info, OctaResult};
use slotmap::Key;

use crate::model::{composer::template::ComposeTemplate, generation::collapse::CollapseOperation};

use super::collapser::{CollapseNodeKey, Collapser};


impl Collapser { 
    pub fn delete_node(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate) {
        let node = self.nodes.remove(node_index);
        if node.is_none() {
            return;
        }
        let node = node.unwrap();
        assert!(!node.defined_by.is_null(), "Trying to delete root node!");

        info!("{:?} Delete node", node_index);

        let template_node = &template.nodes[node.template_index];

        self.pending.delete_collapse(template_node.level, node_index);
        self.pending.delete_create_defined(node_index);

        for (_, depends) in node.depends.iter() {
            for depend in depends {
                let Some(depends_node) = self.nodes.get_mut(*depend) else { 
                    continue;
                };

                let children = depends_node.children.iter_mut()
                    .find(|(template_index, _)| *template_index == node.template_index)
                    .map(|(_, c)| c)
                    .expect("When deleting node the template index of the node was not present in the children of a dependency");

                let i = children.iter()
                    .position(|t| *t == node_index)
                    .expect("When deleting node index of the node was not present in the children of a dependency");

                children.swap_remove(i);
            }
        }

        for child in node.children.iter()
            .map(|(_, c)| c) 
            .flatten() {

            self.delete_node(*child, template);
        }
    } 
}
