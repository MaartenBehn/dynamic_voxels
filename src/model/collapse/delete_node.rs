use octa_force::{anyhow::{self, ensure, anyhow}, log::info, OctaResult};
use slotmap::Key;

use crate::{model::{collapse::collapser::NodeDataType, template::Template}};

use super::collapser::{CollapseNodeKey, Collapser};


impl Collapser { 
    pub fn delete_node(&mut self, node_index: CollapseNodeKey, template: &Template) {
        let node = self.nodes.remove(node_index);
        if node.is_none() {
            return;
        }
        let node = node.unwrap();
        assert!(!node.defined_by.is_null(), "Trying to delete root node!");

        let index_in_template_list = self.nodes_per_template_index[node.template_index].iter().position(|i| *i == node_index).unwrap();
        self.nodes_per_template_index[node.template_index].swap_remove(index_in_template_list);

        #[cfg(debug_assertions)]
        info!("{:?} Delete node", node_index);

        let template_node = &template.nodes[node.template_index];

        self.pending.delete_collapse(template_node.level, node_index);

        for (_, depends) in node.depends.iter() {
            for (depend, _) in depends {
                let Some(depends_node) = self.nodes.get_mut(*depend) else { 
                    continue;
                };

                let (children_index, children) = depends_node.children.iter_mut()
                    .enumerate()
                    .find(|(_, (template_index, _))| *template_index == node.template_index)
                    .map(|(i, (_, c))| (i, c))
                    .expect("When deleting node the template index of the node was not present in the children of a dependency");

                let i = children.iter()
                    .position(|(t, _)| *t == node_index)
                    .expect("When deleting node index of the node was not present in the children of a dependency");

                children.swap_remove(i);

                if children.is_empty() {
                    depends_node.children.swap_remove(children_index);
                }
            }
        }

        for (child, _) in node.children.iter()
            .map(|(_, c)| c) 
            .flatten() {

            self.delete_node(*child, template);
        }
    } 
}
