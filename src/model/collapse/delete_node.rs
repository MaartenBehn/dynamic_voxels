use octa_force::{anyhow::{self, ensure, anyhow}, log::info, OctaResult};
use slotmap::Key;

use crate::{model::{collapse::collapser::NodeDataType, composer::build::{OnDeleteArgs, BS}, template::Template}, util::{number::Nu, vector::Ve}};

use super::collapser::{CollapseNodeKey, Collapser};


impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> { 
    pub fn delete_node(&mut self, node_index: CollapseNodeKey, template: &Template<V2, V3, T, B>, state: &mut B) {
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

        match &node.data {
            NodeDataType::Build(t) => {
                B::on_delete(OnDeleteArgs {
                    collapse_value: t,
                    collapse_node: &node,
                    collapser: &self,
                    template,
                    state,
                })
            },
            _ => {}
        }

        self.pending.delete_collapse(template_node.level, node_index);

        for (_, depends) in node.depends.iter() {
            for depend in depends {
                let Some(depends_node) = self.nodes.get_mut(*depend) else { 
                    continue;
                };

                let (children_index, children) = depends_node.children.iter_mut()
                    .enumerate()
                    .find(|(_, (template_index, _))| *template_index == node.template_index)
                    .map(|(i, (_, c))| (i, c))
                    .expect("When deleting node the template index of the node was not present in the children of a dependency");

                let i = children.iter()
                    .position(|t| *t == node_index)
                    .expect("When deleting node index of the node was not present in the children of a dependency");

                children.swap_remove(i);

                if children.is_empty() {
                    depends_node.children.swap_remove(children_index);
                }
            }
        }

        for child in node.children.iter()
            .map(|(_, c)| c) 
            .flatten() {

            self.delete_node(*child, template, state);
        }
    } 
}
